use crate::card_abstraction::{CardAbstraction, CardAbstractionOptions};
use crate::constants::*;
use crate::game_node::GameNode;
use crate::round::BettingRound;
use crate::sparse_and_dense::{generate_buckets, SparseAndDense};
use crate::tree::Tree;
use crate::tree_builder::{TreeBuilder, TreeBuilderOptions};
use rand::{
    distributions::{Distribution, Uniform},
    prelude::*,
    rngs::{SmallRng, ThreadRng},
    thread_rng, Rng,
};
use rust_poker::hand_evaluator::{evaluate, Hand, CARDS};
use rust_poker::hand_range::{get_card_mask, HandRange};
use rust_poker::HandIndexer;
use std::convert::TryFrom;
use std::iter::FromIterator;
use std::result::Result;
use thiserror::Error as ThisError;

type Error = Box<dyn std::error::Error>;

#[derive(Debug, ThisError)]
pub enum SolverError {
    #[error("too many players")]
    TooManyPlayers,
    #[error("not enough players")]
    TooFewPlayers,
    #[error("invalid board mask")]
    InvalidBoard,
    #[error("array sizes don't match")]
    PlayerCountMismatch,
    #[error("invalid number of rounds in bet sizes")]
    InvalidBetSizes,
    #[error("invalid number of rounds in raise sizes")]
    InvalidRaiseSizes,
    #[error("pot must be greater than zero")]
    InvalidPotSize,
    #[error("invalid abstraction")]
    InvalidCardAbstraction,
}

#[derive(Debug)]
pub struct Infoset {
    regrets: Vec<i32>,
    strategy_sum: Vec<u32>,
}

impl Infoset {
    fn init(n_actions: usize) -> Self {
        Infoset {
            regrets: vec![0; n_actions],
            strategy_sum: vec![0; n_actions],
        }
    }
}

/// options for running a postflop solver simulation
#[derive(Debug)]
pub struct SolverOptions {
    /// initial board mask
    pub board_mask: u64,
    /// hand range for each player
    pub hand_ranges: Vec<String>,
    /// initial stack sizes
    pub stacks: Vec<u32>,
    /// initial pot size
    pub pot: u32,
    /// array of bet sizes for each player for each round
    pub bet_sizes: Vec<Vec<Vec<f64>>>,
    /// array of raise sizes for each player for each round
    pub raise_sizes: Vec<Vec<Vec<f64>>>,
    /// an abstraction for every round
    pub card_abstraction: Vec<String>,
}

impl ToString for SolverOptions {
    fn to_string(&self) -> String {
        let s = format!(
            "b{}-hr{:?}-st{:?}-p{}-bs{:?}-rs{:?}-ca{:?}",
            self.board_mask,
            self.hand_ranges,
            self.stacks,
            self.pot,
            self.bet_sizes,
            self.raise_sizes,
            self.card_abstraction
        );
        s.split_whitespace().collect()
    }
}

#[derive(Debug)]
pub struct Solver {
    /// game tree including all chance, private, and action nodes
    game_tree: Tree<GameNode>,
    /// initial board as 64 bit mask
    initial_board: u64,
    /// hand range for each players
    hand_ranges: Vec<HandRange>,
    /// a card abstraction for each round
    card_abstraction: Vec<CardAbstraction>,
    /// buckets for each player for each round
    buckets: Vec<Vec<SparseAndDense>>,
    /// infoset for each action node index for each bucket
    /// plans to break them up by player and round in the future
    infosets: Vec<Vec<Infoset>>,
    /// random number generator
    rng: ThreadRng,
    /// number of players
    n_players: usize,
    /// hand indexer for each round
    hand_indexers: [HandIndexer; 4],
    /// initial round
    start_round: BettingRound,
    /// number of betting rounds in this tree
    num_rounds: usize,
    /// sampled combo bucket for each player for each round
    combo_buckets: Vec<Vec<usize>>,
    /// score of each hand
    hand_scores: Vec<u16>,
    /// distribution for each hand range
    /// used for sampling
    combo_dists: Vec<Uniform<usize>>,
    /// sampled combo index for this hand
    combo_idxs: Vec<usize>,
}

impl Solver {
    /// initialize a heads up postflop solver using these options
    pub fn init(options: SolverOptions) -> Result<Solver, Error> {
        // check if initial board is valid
        // must be between 3 and 5 cards
        let num_board_cards = options.board_mask.count_ones();
        if options.board_mask >= (1u64 << CARD_COUNT) {
            return Err(SolverError::InvalidBoard.into());
        }
        // number of betting rounds in this tree
        let num_rounds = match num_board_cards {
            3 => 3,
            4 => 2,
            5 => 1,
            _ => {
                return Err(SolverError::InvalidBoard.into());
            }
        };
        let start_round = match num_board_cards {
            3 => BettingRound::FLOP,
            4 => BettingRound::TURN,
            5 => BettingRound::RIVER,
            _ => {
                return Err(SolverError::InvalidBoard.into());
            }
        };

        // check if player count is valid
        let n_players = options.hand_ranges.len();
        if n_players < MIN_PLAYERS {
            return Err(SolverError::TooFewPlayers.into());
        }
        if n_players > MAX_PLAYERS {
            return Err(SolverError::TooManyPlayers.into());
        }
        // check if all argument sizes match
        if options.stacks.len() != n_players {
            return Err(SolverError::PlayerCountMismatch.into());
        }
        if options.bet_sizes.len() != n_players {
            return Err(SolverError::PlayerCountMismatch.into());
        }
        if options.raise_sizes.len() != n_players {
            return Err(SolverError::PlayerCountMismatch.into());
        }
        if options.pot == 0 {
            return Err(SolverError::InvalidPotSize.into());
        }
        // check bet sizes
        for player_bet_sizes in &options.bet_sizes {
            if player_bet_sizes.len() != num_rounds {
                return Err(SolverError::InvalidBetSizes.into());
            }
            for round_bet_sizes in player_bet_sizes {
                for bet_size in round_bet_sizes {
                    if *bet_size < 0.0 {
                        return Err(SolverError::InvalidBetSizes.into());
                    }
                }
            }
        }
        // check raise sizes
        for player_raise_sizes in &options.raise_sizes {
            if player_raise_sizes.len() != num_rounds {
                return Err(SolverError::InvalidRaiseSizes.into());
            }
            for round_raise_sizes in player_raise_sizes {
                for raise_size in round_raise_sizes {
                    if *raise_size < 0.0 {
                        return Err(SolverError::InvalidBetSizes.into());
                    }
                }
            }
        }
        // remove conflicting combos
        let mut hand_ranges: Vec<HandRange> = options
            .hand_ranges
            .iter()
            .map(|rs| HandRange::from_string(rs.to_string()))
            .collect();
        for hr in &mut hand_ranges {
            hr.remove_conflicting_combos(options.board_mask);
        }
        // load card abstraction
        if options.card_abstraction.len() != num_rounds {
            return Err(SolverError::InvalidCardAbstraction.into());
        }
        let card_abstraction: Result<Vec<CardAbstraction>, Error> = options
            .card_abstraction
            .iter()
            .enumerate()
            .map(|(i, abs_string)| {
                let r = BettingRound::try_from(usize::from(start_round) + i)?;
                let ca_opts = CardAbstractionOptions {
                    k: 5000,
                    d: if r == BettingRound::RIVER { 8 } else { 50 },
                    round: r,
                    abs_type: abs_string.to_string(),
                };
                CardAbstraction::load(ca_opts)
            })
            .collect();

        let card_abstraction = match card_abstraction {
            Ok(ca) => ca,
            Err(e) => {
                return Err(e);
            }
        };

        let mut buckets: Vec<Vec<SparseAndDense>> = vec![vec![]];
        for player in 0..n_players {
            buckets.push(Vec::new());
            for round in 0..num_rounds {
                let br = BettingRound::try_from(usize::from(start_round) + round)?;
                let b = generate_buckets(
                    &hand_ranges[player],
                    &card_abstraction[round],
                    br,
                    options.board_mask,
                );
                buckets[player].push(b);
            }
        }

        let tree_options = TreeBuilderOptions {
            blinds: None,
            stacks: options.stacks,
            pot: options.pot,
            round: start_round,
            bet_sizes: options.bet_sizes,
            raise_sizes: options.raise_sizes,
        };
        let game_tree = TreeBuilder::build(&tree_options)?;

        // initialize infosets
        let mut infosets: Vec<Vec<Infoset>> = Vec::new();
        for node in game_tree.iter() {
            if let GameNode::Action {
                actions,
                index,
                round,
                player,
            } = &node.data
            {
                let index = *index as usize;
                if infosets.len() <= index {
                    infosets.resize_with(index + 1, Default::default);
                }
                let n_buckets = buckets[usize::from(*player)][usize::from(*round)].len();
                let n_actions = actions.len();

                infosets[index] = (0..n_buckets).map(|_| Infoset::init(n_actions)).collect();
            }
        }
        // create hand indexers
        let hand_indexers = [
            HandIndexer::init(1, vec![2]),
            HandIndexer::init(2, vec![2, 3]),
            HandIndexer::init(2, vec![2, 4]),
            HandIndexer::init(2, vec![2, 5]),
        ];
        // create combo buckets
        let combo_buckets = vec![vec![0; num_rounds]; n_players];
        // create hand scores
        let hand_scores = vec![0; n_players];
        // create combo idxs
        let combo_idxs = vec![0usize; n_players];
        // create combo distributions for sampling
        let combo_dists = hand_ranges
            .iter()
            .map(|hr| Uniform::from(0..hr.hands.len()))
            .collect();

        let solver = Solver {
            game_tree,
            initial_board: options.board_mask,
            card_abstraction,
            buckets,
            hand_ranges,
            infosets,
            rng: thread_rng(),
            n_players,
            hand_indexers,
            start_round,
            num_rounds,
            combo_buckets,
            combo_dists,
            combo_idxs,
            hand_scores,
        };
        Ok(solver)
    }
    /// deal a hand and get player reach indexes, winning player, and buckets
    pub fn deal(&mut self) {
        // select a hole card pair
        let mut used_card_mask = self.initial_board;
        for (player, hr) in self.hand_ranges.iter().enumerate() {
            loop {
                let combo_idx = self.combo_dists[player].sample(&mut self.rng);
                let combo_mask = (1u64 << hr.hands[combo_idx].0) | (1u64 << hr.hands[combo_idx].1);
                if (combo_mask & used_card_mask) == 0 {
                    self.combo_idxs[player] = combo_idx;
                    used_card_mask |= combo_mask;
                    break;
                }
            }
        }
        // copy board cards
        let mut cards = [52u8; 7];
        let mut i = 2;
        for j in 0..CARD_COUNT {
            if (self.initial_board & (1u64 << j)) != 0 {
                cards[i] = j;
                i += 1;
            }
        }
        // generate remaining board cards
        for j in i..7 {
            loop {
                let c = self.rng.gen_range(0, CARD_COUNT);
                if ((1u64 << c) & used_card_mask) == 0 {
                    cards[j] = c;
                    used_card_mask |= 1u64 << c;
                    break;
                }
            }
        }
        // score each hands
        let mut board: Hand = Hand::default();
        for c in &cards[2..7] {
            board += CARDS[usize::from(*c)];
        }
        // generate combo buckes and score hands
        for player in 0..self.n_players {
            let hole_cards = &self.hand_ranges[player].hands[self.combo_idxs[player]];
            cards[0] = hole_cards.0;
            cards[1] = hole_cards.1;
            let hand = board + CARDS[usize::from(cards[0])] + CARDS[usize::from(cards[1])];
            self.hand_scores[player] = evaluate(&hand);
            for round in 0..self.num_rounds {
                let cannon_idx = self.hand_indexers[usize::from(self.start_round) + round]
                    .get_index(&cards) as usize;
                let bucket_idx = self.card_abstraction[round].get(cannon_idx) as usize;
                let dense_idx = self.buckets[player][round]
                    .sparse_to_dense(bucket_idx)
                    .unwrap();
                self.combo_buckets[player][round] = dense_idx;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::bench::Bencher;

    #[test]
    fn test_init_flop_valid() {
        let options = SolverOptions {
            board_mask: get_card_mask("AhAdAc"),
            hand_ranges: vec!["random".to_string(), "random".to_string()],
            stacks: vec![10000, 10000],
            pot: 100,
            bet_sizes: vec![
                vec![vec![0.5], vec![0.5], vec![0.5]],
                vec![vec![0.5], vec![0.5], vec![0.5]],
            ],
            raise_sizes: vec![
                vec![vec![1.0], vec![1.0], vec![1.0]],
                vec![vec![1.0], vec![1.0], vec![1.0]],
            ],
            card_abstraction: vec!["null".to_string(), "emd".to_string(), "ochs".to_string()],
        };
        let solver = Solver::init(options);
        assert_eq!(solver.is_ok(), true);
    }

    #[test]
    fn test_init_turn_valid() {
        let options = SolverOptions {
            board_mask: get_card_mask("AhAdAcKc"),
            hand_ranges: vec!["random".to_string(), "random".to_string()],
            stacks: vec![10000, 10000],
            pot: 100,
            bet_sizes: vec![vec![vec![0.5], vec![0.5]], vec![vec![0.5], vec![0.5]]],
            raise_sizes: vec![vec![vec![1.0], vec![1.0]], vec![vec![1.0], vec![1.0]]],
            card_abstraction: vec!["null".to_string(), "ochs".to_string()],
        };
        let solver = Solver::init(options);
        assert_eq!(solver.is_ok(), true);
    }

    #[test]
    fn test_init_river_valid() {
        let options = SolverOptions {
            board_mask: get_card_mask("AhAdAcKc2d"),
            hand_ranges: vec!["random".to_string(), "random".to_string()],
            stacks: vec![10000, 10000],
            pot: 100,
            bet_sizes: vec![vec![vec![0.5]], vec![vec![0.5]]],
            raise_sizes: vec![vec![vec![1.0]], vec![vec![1.0]]],
            card_abstraction: vec!["null".to_string()],
        };
        let solver = Solver::init(options);
        assert_eq!(solver.is_ok(), true);
    }

    #[bench]
    // 1,062 ns/iter (+/- 62)
    fn bench_deal(b: &mut Bencher) {
        let options = SolverOptions {
            board_mask: get_card_mask("AhAdAc"),
            hand_ranges: vec!["random".to_string(), "random".to_string()],
            stacks: vec![10000, 10000],
            pot: 100,
            bet_sizes: vec![
                vec![vec![0.5], vec![0.5], vec![0.5]],
                vec![vec![0.5], vec![0.5], vec![0.5]],
            ],
            raise_sizes: vec![
                vec![vec![1.0], vec![1.0], vec![1.0]],
                vec![vec![1.0], vec![1.0], vec![1.0]],
            ],
            card_abstraction: vec!["null".to_string(), "emd".to_string(), "ochs".to_string()],
        };
        let mut solver = Solver::init(options).unwrap();
        b.iter(|| {
            solver.deal();
        });
    }

    #[test]
    fn test_options_to_string() {
        let options = SolverOptions {
            board_mask: get_card_mask("AhAdAcKc"),
            hand_ranges: vec!["random".to_string(), "random".to_string()],
            stacks: vec![10000, 10000],
            pot: 100,
            bet_sizes: vec![vec![vec![0.5], vec![0.5]], vec![vec![0.5], vec![0.5]]],
            raise_sizes: vec![vec![vec![1.0], vec![1.0]], vec![vec![1.0], vec![1.0]]],
            card_abstraction: vec!["null".to_string(), "ochs".to_string()],
        };
        let as_string = "b4081387162304512-hr[\"random\",\"random\"]-st[10000,10000]-p100-bs[[[0.5],[0.5]],[[0.5],[0.5]]]-rs[[[1.0],[1.0]],[[1.0],[1.0]]]-ca[\"null\",\"ochs\"]";
        assert_eq!(&options.to_string(), as_string);
    }
}
