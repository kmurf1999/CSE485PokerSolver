use crate::action::Action;
use crate::betting_abstraction::BettingAbstraction;
use crate::card::Card;
use crate::card_abstraction::{CardAbstraction, CardAbstractionOptions};
use crate::infoset::{sample_action_index, InfosetTable};
use crate::round::BettingRound;
use crate::state::GameState;
use rand::prelude::SmallRng;
use rand::Rng;
use rand::SeedableRng;

use rust_poker::hand_range::HandRange;
use rust_poker::HandIndexer;
use std::convert::TryFrom;
use std::result::Result;
use thiserror::Error as ThisError;

type Error = Box<dyn std::error::Error>;

// const PRUNE_THRESHOLD: f64 = -300_000_000.0;
// const STRATEGY_INTERVAL: usize = 10_000;
// const DISCOUNT_INTERVAL: usize = 10_000_000;

#[derive(Debug, ThisError)]
pub enum SolverError {
    #[error("invalid board")]
    InvalidBoard,
    #[error("invalid hand range")]
    InvalidHandRange,
    #[error("invalid card abstraction")]
    InvalidCardAbstraction,
}

/// options for running a postflop solver simulation
#[derive(Debug)]
pub struct SolverOptions {
    /// initial state of the game
    pub initial_state: GameState,
    /// hand range for each player
    pub hand_ranges: [String; 2],
    /// A betting abstraction for each player
    pub betting_abstraction: BettingAbstraction,
    /// an abstraction for every round
    pub card_abstraction: Vec<String>,
}

#[derive(Debug)]
pub struct Solver {
    /// A hand range for each player
    pub hand_ranges: Vec<HandRange>,
    /// The initial game state of the game
    pub initial_state: GameState,
    /// An table of information sets for storing regrets and strategys
    pub infosets: InfosetTable,
    /// Restricts avaible bet and raise sizes
    pub betting_abstraction: BettingAbstraction,
    /// Used to map hand idx -> bucket idx
    /// One for each round
    card_abstraction: [CardAbstraction; 4],
    /// Hand indexers
    /// One for each round
    hand_indexers: [HandIndexer; 4],
}

impl Solver {
    /// Initialize a solver
    pub fn init(options: SolverOptions) -> Result<Solver, Error> {
        let board = options.initial_state.board();
        let mut board_card_count = 0;
        let mut board_mask = 0u64;
        for card in board {
            if *card >= 52 {
                break;
            }
            board_mask |= 1u64 << *card;
            board_card_count += 1;
        }
        let num_rounds = match board_card_count {
            0 => 4,
            3 => 3,
            4 => 2,
            5 => 1,
            _ => {
                return Err(SolverError::InvalidBoard.into());
            }
        };
        let start_round = 4 - num_rounds;
        let mut hand_ranges = HandRange::from_strings(options.hand_ranges.to_vec());
        for hr in &mut hand_ranges {
            hr.remove_conflicting_combos(board_mask);
            if hr.hands.is_empty() {
                return Err(SolverError::InvalidHandRange.into());
            }
        }
        if options.card_abstraction.len() != num_rounds {
            return Err(SolverError::InvalidCardAbstraction.into());
        }
        let mut card_abstraction = [
            CardAbstraction::default(),
            CardAbstraction::default(),
            CardAbstraction::default(),
            CardAbstraction::default(),
        ];
        for round in 0..num_rounds {
            let r = match BettingRound::try_from(start_round + round) {
                Ok(r) => r,
                Err(e) => {
                    return Err(e.into());
                }
            };
            let ca_opts = CardAbstractionOptions {
                round: r,
                abs_type: options.card_abstraction[round].to_string(),
                k: 5000,
                d: if options.card_abstraction[round] == "emd" {
                    50
                } else {
                    8
                },
            };
            card_abstraction[usize::from(r)] = CardAbstraction::load(ca_opts)?;
        }
        let hand_indexers = [
            HandIndexer::init(1, [2].to_vec()),
            HandIndexer::init(2, [2, 3].to_vec()),
            HandIndexer::init(2, [2, 4].to_vec()),
            HandIndexer::init(2, [2, 5].to_vec()),
        ];
        Ok(Solver {
            hand_ranges,
            initial_state: options.initial_state,
            infosets: InfosetTable::default(),
            betting_abstraction: options.betting_abstraction,
            hand_indexers,
            card_abstraction,
        })
    }
    /// Run MCCFR for `iterations` iterations
    /// returns the average equity/iter for each player
    pub fn run(&mut self, iterations: usize) -> [f64; 2] {
        let mut rng = SmallRng::from_entropy();
        let mut equity = [0f64; 2];
        for _ in 0..iterations {
            for player in 0..2 {
                equity[usize::from(player)] +=
                    self.traverse(self.initial_state.clone(), player, &mut rng);
            }
        }
        for eq in &mut equity {
            *eq /= 0.5 * iterations as f64;
        }
        equity
    }
    /// MCCFR implementation
    fn traverse<R: Rng>(&mut self, node: GameState, player: u8, rng: &mut R) -> f64 {
        // return the reward for this player
        if node.is_terminal() {
            return node.player_reward(usize::from(player));
        }
        // sample a single chance outcome
        if node.is_chance() {
            let action: Action;
            if node.is_initial_state() {
                action = node.sample_private_chance_from_ranges(rng, &self.hand_ranges);
            } else {
                action = node.sample_public_chance(rng);
            }
            return self.traverse(node.apply_action(action), player, rng);
        }
        let legal_actions = node.valid_actions(&self.betting_abstraction);
        let action_count = legal_actions.len();
        let hole_cards = node.acting_player().cards();
        let hand_bucket = self.get_bucket(hole_cards[0], hole_cards[1], &node);
        let is_key = format!("{}{}", hand_bucket, node.history_string());
        let current_strategy = self
            .infosets
            .get_or_insert(is_key.clone(), action_count)
            .current_strategy();
        let mut node_util = 0f64;
        let mut child_utils;
        if node.current_player() == player as i8 {
            child_utils = vec![0f64; action_count];
            for a_idx in 0..action_count {
                child_utils[a_idx] =
                    self.traverse(node.apply_action(legal_actions[a_idx]), player, rng);
                node_util += current_strategy[a_idx] * child_utils[a_idx];
            }
        } else {
            child_utils = vec![];
            let a_idx = sample_action_index(&current_strategy, rng);
            node_util = self.traverse(node.apply_action(legal_actions[a_idx]), player, rng);
        }

        let infoset = self.infosets.get_or_insert(is_key, action_count);
        if node.current_player() == player as i8 {
            // update regrets
            for a_idx in 0..action_count {
                infoset.cummulative_regrets[a_idx] += child_utils[a_idx] - node_util;
            }
        } else {
            // update avg strateg
            for a_idx in 0..action_count {
                infoset.cummulative_strategy[a_idx] += current_strategy[a_idx];
            }
        }
        node_util
    }
    /// Discount all infosets using LCFR
    pub fn discount(&mut self, t: usize) {
        let discount_factor = (t as f64) / (1.0 + t as f64);
        for (_, infoset) in self.infosets.table.iter_mut() {
            infoset
                .cummulative_regrets
                .iter_mut()
                .for_each(|val| *val *= discount_factor);
            infoset
                .cummulative_strategy
                .iter_mut()
                .for_each(|val| *val *= discount_factor);
        }
    }
    /// get bucket from hole cards
    pub fn get_bucket(&self, c1: u8, c2: u8, node: &GameState) -> u32 {
        let mut cards: [Card; 7] = [52; 7];
        cards[0] = c1;
        cards[1] = c2;
        cards[2..(5 + 2)].clone_from_slice(&node.board()[..5]);
        let hand_index = self.hand_indexers[usize::from(node.round())].get_index(&cards);
        let hand_bucket = self.card_abstraction[usize::from(node.round())].get(hand_index as usize);
        hand_bucket as u32
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::state::GameStateOptions;

    #[test]
    fn test_solver_init() -> Result<(), Box<dyn std::error::Error>> {
        // basic river solver
        let initial_state = GameState::new(GameStateOptions {
            stacks: [10000, 10000],
            initial_board: [0, 1, 2, 3, 4],
            wagers: [0, 0],
            pot: 1000,
        })?;
        let betting_abstraction = BettingAbstraction {
            bet_sizes: [vec![], vec![], vec![], vec![0.5, 1.0]],
            raise_sizes: [vec![], vec![], vec![], vec![1.0]],
            all_in_threshold: 0f64,
        };
        let solver = Solver::init(SolverOptions {
            initial_state,
            hand_ranges: [String::from("random"), String::from("random")],
            betting_abstraction,
            card_abstraction: vec![String::from("null")],
        });
        assert_eq!(solver.is_ok(), true);
        Ok(())
    }
}
