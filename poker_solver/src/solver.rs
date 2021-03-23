use crate::card_abstraction::{CardAbstraction, CardAbstractionOptions};
use crate::constants::*;
use crate::game_node::{GameNode, TerminalType};
use crate::round::BettingRound;
use crate::sparse_and_dense::{generate_buckets, SparseAndDense};
use crate::tree::Tree;
use crate::tree_builder::{TreeBuilder, TreeBuilderOptions};
use rand::{
    distributions::{Distribution, Uniform, WeightedIndex},
    prelude::*,
    rngs::{SmallRng, ThreadRng},
    thread_rng, Rng,
};
use rust_poker::hand_evaluator::{evaluate, Hand, CARDS};
use rust_poker::hand_range::{get_card_mask, HandRange};
use rust_poker::read_write::VecIO;
use rust_poker::HandIndexer;
use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::iter::FromIterator;
use std::mem::{forget, size_of, transmute};
use std::result::Result;
use std::slice;
use thiserror::Error as ThisError;

type Error = Box<dyn std::error::Error>;
const PRUNE_THRESHOLD: i32 = -300_000_000;
const STRATEGY_INTERVAL: usize = 10000;

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
    pub regrets: Vec<i32>,
    pub strategy_sum: Vec<i32>,
}

/// Reads a Vector of type T from file
/// todo fix this function
fn read_vec_from_file<T>(file: &mut File, size: usize) -> Result<Vec<T>, Error> {
    let mut buffer: Vec<T> = Vec::new();
    unsafe {
        let mut converted = Vec::<u8>::from_raw_parts(
            buffer.as_mut_ptr() as *mut u8,
            size * size_of::<T>(),
            size * size_of::<T>(),
        );
        match file.read_to_end(&mut converted) {
            Ok(s) => {
                if converted.len() % size_of::<T>() != 0 {
                    converted.truncate(size * size_of::<T>());
                    forget(converted);
                    // todo
                    return Err(SolverError::TooManyPlayers.into());
                }
            }
            Err(e) => {
                converted.truncate(size * size_of::<T>());
                forget(converted);
                return Err(e.into());
            }
        }
        let new_length = converted.len() / size_of::<T>();
        let new_capacity = converted.len() / size_of::<T>();
        buffer = Vec::from_raw_parts(converted.as_mut_ptr() as *mut T, new_length, new_capacity);
        forget(converted);
        Ok(buffer)
    }
}

#[inline(always)]
fn sample_pdf<R: Rng>(pdf: &Vec<f64>, rng: &mut R) -> usize {
    let rand = rng.gen_range(0.0, 1.0);
    let mut s = 0.0;
    for i in 0..pdf.len() {
        s += pdf[i];
        if rand < s {
            return i;
        }
    }
    pdf.len() - 1
}

impl Default for Infoset {
    fn default() -> Infoset {
        Infoset {
            regrets: Vec::new(),
            strategy_sum: Vec::new(),
        }
    }
}

impl Infoset {
    pub fn init(n_actions: usize, n_buckets: usize) -> Self {
        Infoset {
            regrets: vec![0; n_actions * n_buckets],
            strategy_sum: vec![0; n_actions * n_buckets],
        }
    }
    // get current strategy through regret-matching
    pub fn current_strategy(&self, bucket: usize, n_actions: usize) -> Vec<f64> {
        let offset = bucket * n_actions;
        let regrets = &self.regrets[offset..offset + n_actions];
        let mut strategy = vec![0f64; n_actions];
        let mut norm_sum = 0f64;
        for a in 0..n_actions {
            strategy[a] = if self.regrets[a] > 0 {
                self.regrets[a] as f64
            } else {
                0.0
            };
            norm_sum += strategy[a];
        }
        for a in 0..n_actions {
            if norm_sum > 0.0 {
                strategy[a] /= norm_sum;
            } else {
                strategy[a] = 1.0 / n_actions as f64;
            }
        }
        strategy
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
        s.replace(&[' ', '\"'][..], "")
    }
}

#[derive(Debug)]
pub struct Solver {
    options_string: String,
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
    infosets: Vec<Infoset>,
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
    winner_mask: u8,
    winner_count: u8,
    /// distribution for each hand range
    /// used for sampling
    combo_dists: Vec<Uniform<usize>>,
    /// sampled combo index for this hand
    combo_idxs: Vec<usize>,
}

impl Solver {
    /// initialize a heads up postflop solver using these options
    pub fn init(options: SolverOptions) -> Result<Solver, Error> {
        // used for filenames
        let options_string = options.to_string();
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
        let mut infosets: Vec<Infoset> = Vec::new();
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

                infosets[index] = Infoset::init(n_actions, n_buckets);
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
            options_string,
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
            winner_count: 0,
            winner_mask: 0,
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
        self.winner_mask = 0u8;
        self.winner_count = 0u8;
        let mut high_score = 0u16;
        for i in 0..self.n_players {
            match self.hand_scores[i].cmp(&high_score) {
                Ordering::Greater => {
                    self.winner_mask = 1u8 << i;
                    self.winner_count = 1;
                    high_score = self.hand_scores[i];
                }
                Ordering::Equal => {
                    self.winner_mask |= 1u8 << i;
                    self.winner_count += 1;
                }
                _ => {}
            }
        }
    }
    fn update_strategy(&mut self) {
        // for node in self.game_tree.iter() {
        //     if let GameNode::Action {
        //         index,
        //         player: _,
        //         actions: _,
        //         round: _,
        //     } = &node.data
        //     {
        //         for infoset in &mut self.infosets[*index as usize] {
        //             let s = infoset.current_strategy();
        //             let a = sample_pdf(&s, &mut self.rng);
        //             infoset.strategy_sum[a] += 1;
        //         }
        //     }
        // }
    }
    fn traverse(&self, node_idx: usize, cfr_reach: f64, player: u8, prune: bool) -> f64 {
        let node = self.game_tree.get_node(node_idx);
        match &node.data {
            GameNode::Terminal {
                value,
                ttype,
                last_to_act,
            } => {
                if let TerminalType::Fold = ttype {
                    if *last_to_act == player {
                        -1.0 * (*value as f64)
                    } else {
                        *value as f64
                    }
                } else {
                    if ((1u8 << player) & self.winner_mask) != 0 {
                        return (*value as f64) / (self.winner_count as f64);
                    }
                    -1.0 * (*value as f64 / (self.winner_count as f64))
                }
            }
            GameNode::Action {
                round,
                index,
                player: node_player,
                actions,
            } => {
                // if we are the player acting
                let n_actions = actions.len();
                let bucket = self.combo_buckets[usize::from(*node_player)][usize::from(*round)];
                let offset = n_actions * bucket;
                let strategy = self.infosets[*index as usize].current_strategy(bucket, n_actions);
                if *node_player == player {
                    let mut node_util = 0f64;
                    let mut action_utils = vec![0f64; n_actions];
                    let mut explored = vec![false; n_actions];
                    if prune {
                        let regrets =
                            &self.infosets[*index as usize].regrets[offset..offset + n_actions];
                        for a in 0..n_actions {
                            let child_is_terminal = matches!(
                                &self.game_tree.get_node(node.children[a]).data,
                                GameNode::Terminal {
                                    last_to_act: _,
                                    value: _,
                                    ttype: _
                                }
                            );
                            if child_is_terminal || regrets[a] > PRUNE_THRESHOLD {
                                explored[a] = true;
                                action_utils[a] =
                                    self.traverse(node.children[a], cfr_reach, player, prune);
                                node_util += action_utils[a] * strategy[a];
                            }
                        }
                    } else {
                        for a in 0..n_actions {
                            action_utils[a] =
                                self.traverse(node.children[a], cfr_reach, player, prune);
                            node_util += action_utils[a] * strategy[a];
                        }
                    }
                    unsafe {
                        let mut regrets =
                            (&self.infosets[*index as usize].regrets[offset..offset + n_actions]
                                as *const [i32]) as *mut [i32];
                        for a in 0..n_actions {
                            if !prune || explored[a] {
                                (&mut *regrets)[a] += (action_utils[a] - node_util) as i32;
                            }
                        }
                    }
                    return node_util;
                }
                let mut rng = SmallRng::from_entropy();
                let sampled_action = sample_pdf(&strategy, &mut rng);
                self.traverse(
                    node.children[sampled_action],
                    cfr_reach * strategy[sampled_action],
                    player,
                    prune,
                )
            }
            _ => self.traverse(node.children[0], cfr_reach, player, prune),
        }
    }
    pub fn run(&mut self, max_iterations: usize) {
        let mut evs = vec![0.0; self.n_players];
        for t in 1..max_iterations {
            if (t % STRATEGY_INTERVAL) == 0 {
                self.update_strategy();
            }
            for p in 0..self.n_players {
                self.deal();
                // TODO start cfr reach with product of actual combo weights
                let prune = self.rng.gen_range(0.0, 1.0) < 0.05;
                evs[p] += self.traverse(0, 1.0, p as u8, prune);
            }
        }
        println!("{:?}", evs);
    }
    /// save regrets to a file
    fn save_regrets(&self) -> Result<(), Error> {
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(format!("data/regrets-{}.dat", self.options_string))?;
        for action_index in 0..self.infosets.len() {
            file.write_slice_to_file(&self.infosets[action_index].regrets[..])?;
        }
        Ok(())
    }
    /// load regrets from a file
    fn load_regrets(&mut self) -> Result<(), Error> {
        let mut file = OpenOptions::new()
            .read(true)
            .open(format!("data/regrets-{}.dat", self.options_string))?;
        let mut offset = 0u64;
        for action_index in 0..self.infosets.len() {
            file.seek(SeekFrom::Start(offset))?;
            let size = self.infosets[action_index].regrets.len();
            let mut buffer = vec![0; size * size_of::<i32>()];
            file.read(&mut buffer)?;
            unsafe {
                self.infosets[action_index].regrets =
                    Vec::from_raw_parts(buffer.as_mut_ptr() as *mut i32, size, size);
                forget(buffer);
            }
            offset += (self.infosets[action_index].regrets.len() * size_of::<i32>()) as u64;
        }
        Ok(())
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

    #[bench]
    // 18,400 ns/iter (+/- 7,624)
    fn bench_traverse_prune(b: &mut Bencher) {
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
        solver.deal();
        b.iter(|| {
            solver.traverse(0, 1.0, 0, true);
        });
    }

    #[bench]
    // 38,969,912 ns/iter (+/- 14,411,448)
    fn bench_train_1000(b: &mut Bencher) {
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
            solver.run(1000);
        });
    }

    #[test]
    fn test_train_100000() {
        let options = SolverOptions {
            board_mask: get_card_mask("QhJdTs"),
            hand_ranges: vec![
                "22+,AT+,KT+,QT+,JTs,A5s".to_string(),
                "22+,A9+,KT+,QT+,JT+,T9s,98s,87s,".to_string(),
            ],
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
        solver.run(100000);
    }

    #[test]
    fn test_save_load() {
        let options = SolverOptions {
            board_mask: get_card_mask("QhJdTs"),
            hand_ranges: vec![
                "22+,AT+,KT+,QT+,JTs,A5s".to_string(),
                "22+,A9+,KT+,QT+,JT+,T9s,98s,87s,".to_string(),
            ],
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
        solver.run(10000);
        // generate test data
        let mut rng = thread_rng();
        let mut test_data = vec![];
        for _ in 0..1000 {
            let action_idx = rng.gen_range(0, solver.infosets.len());
            let idx = rng.gen_range(0, solver.infosets[action_idx].regrets.len());
            let value = solver.infosets[action_idx].regrets[idx];
            test_data.push((action_idx, idx, value));
        }
        assert_eq!(solver.save_regrets().is_ok(), true);
        assert_eq!(solver.load_regrets().is_ok(), true);
        // compare test data to original
        for (action_idx, idx, value) in test_data {
            assert_eq!(solver.infosets[action_idx].regrets[idx], value);
        }
    }

    #[bench]
    // 10,274,775 ns/iter (+/- 1,330,963)
    fn bench_update_strategy(b: &mut Bencher) {
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
            solver.update_strategy();
        });
    }

    #[test]
    fn test_sample_pdf() {
        let mut rng = thread_rng();
        let pdf = vec![1.0, 0.0, 0.0];
        let a = sample_pdf(&pdf, &mut rng);
        assert_eq!(a, 0);
        let pdf = vec![0.0, 1.0, 0.0];
        let a = sample_pdf(&pdf, &mut rng);
        assert_eq!(a, 1);
        let pdf = vec![0.0, 0.0, 1.0];
        let a = sample_pdf(&pdf, &mut rng);
        assert_eq!(a, 2);
        let pdf = vec![0.0, 0.5, 0.5];
        let a = sample_pdf(&pdf, &mut rng);
        assert_ne!(a, 0);
        let pdf = vec![0.5, 0.5, 0.0];
        let a = sample_pdf(&pdf, &mut rng);
        assert_ne!(a, 2);
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
        let as_string = "b4081387162304512-hr[random,random]-st[10000,10000]-p100-bs[[[0.5],[0.5]],[[0.5],[0.5]]]-rs[[[1.0],[1.0]],[[1.0],[1.0]]]-ca[null,ochs]";
        assert_eq!(&options.to_string(), as_string);
    }
}
