use crate::action::{Action, CALL_IDX, CHECK_FOLD_IDX};
use crate::betting_abstraction::BettingAbstraction;
use crate::card::Card;
use crate::card_abstraction::{CardAbstraction, CardAbstractionOptions};
// use crate::constants::*;
use crate::combos::CardComboIter;
use crate::infoset::{sample_action_index, InfosetTable};
use crate::normalize;
use crate::round::BettingRound;
use crate::state::GameState;
use rand::{rngs::ThreadRng, thread_rng};
use rust_poker::hand_evaluator::{evaluate, Hand, CARDS};
use rust_poker::hand_range::{Combo, HandRange};
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
    hand_ranges: Vec<HandRange>,
    /// The initial game state of the game
    initial_state: GameState,
    /// An table of information sets for storing regrets and strategys
    pub infosets: InfosetTable,
    /// Restricts avaible bet and raise sizes
    betting_abstraction: BettingAbstraction,
    /// Used to map hand idx -> bucket idx
    /// One for each round
    card_abstraction: Vec<CardAbstraction>,
    /// Hand indexers
    /// One for each round
    hand_indexers: [HandIndexer; 4],
    /// An rng used for sampling actions
    rng: ThreadRng,
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
        let mut card_abstraction = Vec::new();
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
            card_abstraction.push(CardAbstraction::load(ca_opts)?);
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
            rng: thread_rng(),
        })
    }
    /// Run MCCFR for `iterations` iterations
    /// returns the average equity/iter for each player
    pub fn run(&mut self, iterations: usize) -> [f64; 2] {
        let mut equity = [0f64; 2];
        for i in 0..iterations {
            for player in 0..2 {
                equity[usize::from(player)] += self.traverse(self.initial_state.clone(), player);
            }
        }
        for eq in &mut equity {
            *eq /= iterations as f64;
        }
        equity
    }
    pub fn run_br(&mut self, iterations: usize, br_player: u8) -> f64 {
        let mut equity = 0f64;
        let mut beliefs = vec![0f64; self.hand_ranges[usize::from(1 - br_player)].hands.len()];
        for (i, combo) in self.hand_ranges[usize::from(1 - br_player)]
            .hands
            .iter()
            .enumerate()
        {
            beliefs[i] = (combo.2 as f64) / 100f64;
        }
        for _ in 0..iterations {
            equity += self.traverse_br(
                self.initial_state.clone(),
                &mut beliefs.to_owned(),
                br_player,
            );
        }
        equity /= iterations as f64;
        equity
    }
    /// MCCFR implementation
    fn traverse(&mut self, node: GameState, player: u8) -> f64 {
        // return the reward for this player
        if node.is_terminal() {
            return node.player_reward(usize::from(player));
        }
        // sample a single chance outcome
        if node.is_chance() {
            let action: Action;
            if node.is_initial_state() {
                action = node.sample_private_chance_from_ranges(&mut self.rng, &self.hand_ranges);
            } else {
                action = node.sample_public_chance(&mut self.rng);
            }
            return self.traverse(node.apply_action(action), player);
        }
        let legal_actions = node.valid_actions(&self.betting_abstraction);
        let action_count = legal_actions.len();
        let mut cards: [Card; 7] = [52; 7];
        cards[..2].clone_from_slice(&node.player(usize::from(player)).cards()[..]);
        cards[2..(5 + 2)].clone_from_slice(&node.board()[..5]);
        // TODO fix this in the future to actaully use card abstraction
        let hand_bucket = self.hand_indexers[usize::from(node.round())].get_index(&cards);
        let is_key = format!("{}{}", hand_bucket, node.history_string());
        let current_strategy = self
            .infosets
            .get_or_insert(is_key.clone(), action_count)
            .current_strategy();
        let mut node_util = 0f64;
        let mut child_utils = Vec::new();
        if node.current_player() == player as i8 {
            for a_idx in 0..action_count {
                child_utils.push(self.traverse(node.apply_action(legal_actions[a_idx]), player));
                node_util += current_strategy[a_idx] * child_utils[a_idx];
            }
        } else {
            let a_idx = sample_action_index(&current_strategy, &mut self.rng);
            node_util += self.traverse(node.apply_action(legal_actions[a_idx]), player);
        }

        let infoset = self.infosets.get_or_insert(is_key, action_count);
        if node.current_player() == player as i8 {
            // update regrets
            for a_idx in 0..action_count {
                infoset.cummulative_regrets[a_idx] += child_utils[a_idx] - node_util;
            }
        } else {
        } // update avg strategy
        for a_idx in 0..action_count {
            infoset.cummulative_strategy[a_idx] += current_strategy[a_idx];
        }
        node_util
    }
    fn traverse_br(&mut self, node: GameState, beliefs: &mut Vec<f64>, br_player: u8) -> f64 {
        // return the reward for this player
        if node.is_terminal() {
            return node.player_reward(usize::from(br_player));
        }
        // sample a single chance outcome
        if node.is_chance() {
            let action: Action;
            if node.is_initial_state() {
                action = node.sample_private_chance_from_ranges(&mut self.rng, &self.hand_ranges);
                // update beliefs
                if let Action::Chance(cards) = action {
                    let our_card_mask = (1u64 << cards[usize::from(2 * br_player)])
                        | (1u64 << cards[usize::from(2 * br_player + 1)]);
                    for (i, combo) in self.hand_ranges[usize::from(1 - br_player)]
                        .hands
                        .iter()
                        .enumerate()
                    {
                        if beliefs[i] == 0f64 {
                            continue;
                        }
                        let combo_mask = (1u64 << combo.0) | (1u64 << combo.1);
                        if (our_card_mask & combo_mask) != 0 {
                            beliefs[i] = 0f64;
                        }
                    }
                }
            } else {
                action = node.sample_public_chance(&mut self.rng);
                // update beliefs
                if let Action::Chance(cards) = action {
                    let mut board_mask = 0u64;
                    for card in &cards {
                        if *card >= 52 {
                            break;
                        }
                        board_mask |= 1u64 << *card;
                    }
                    for card in node.board() {
                        if *card >= 52 {
                            break;
                        }
                        board_mask |= 1u64 << *card;
                    }
                    for (i, combo) in self.hand_ranges[usize::from(1 - br_player)]
                        .hands
                        .iter()
                        .enumerate()
                    {
                        if beliefs[i] == 0f64 {
                            continue;
                        }
                        let combo_mask = (1u64 << combo.0) | (1u64 << combo.1);
                        if (board_mask & combo_mask) != 0 {
                            beliefs[i] = 0f64;
                        }
                    }
                }
            }
            return self.traverse_br(node.apply_action(action), beliefs, br_player);
        }
        if node.current_player() == br_player as i8 {
            // calculate action using br
            let action = self.local_br(node.clone(), beliefs, br_player);
            self.traverse_br(node.apply_action(action), beliefs, br_player)
        } else {
            // calculate action using cummulative strategy
            let legal_actions = node.valid_actions(&self.betting_abstraction);
            let action_count = legal_actions.len();
            let mut cards: [Card; 7] = [52; 7];
            cards[..2].clone_from_slice(&node.player(usize::from(br_player)).cards()[..]);
            cards[2..(5 + 2)].clone_from_slice(&node.board()[..5]);
            let hand_bucket = self.hand_indexers[usize::from(node.round())].get_index(&cards);
            let is_key = format!("{}{}", hand_bucket, node.history_string());
            let strategy = self
                .infosets
                .get_or_insert(is_key.clone(), action_count)
                .average_strategy();
            let a_idx = sample_action_index(&strategy, &mut self.rng);
            self.traverse_br(node.apply_action(legal_actions[a_idx]), beliefs, br_player)
        }
    }
    /// Calculates best response action using local br function
    fn local_br(&mut self, node: GameState, beliefs: &Vec<f64>, br_player: u8) -> Action {
        let valid_actions = node.valid_actions(&self.betting_abstraction);
        let mut action_utils = vec![0f64; valid_actions.len()];
        // get check/call util
        let wp = self.wp_rollout(node.clone(), &beliefs, br_player);
        let pot = node.pot() as f64;
        let asked = (node.player(usize::from(1 - br_player)).wager()
            - node.player(usize::from(br_player)).wager()) as f64;
        // the utility for check/call
        action_utils[CALL_IDX] = (wp * pot) - ((1f64 - wp) * asked);
        // loop over bet / raises
        for (i, action) in valid_actions.iter().enumerate() {
            // continue if action is not bet/size
            let br_amt: f64;
            if let Action::BetRaise(amt) = action {
                br_amt = *amt as f64;
            } else {
                continue;
            };
            let mut fp = 0f64;
            let next_state = node.apply_action(*action);
            let opp_legal_actions = next_state.valid_actions(&self.betting_abstraction);
            let opp_action_count = opp_legal_actions.len();
            // loop over opponent range
            let mut new_beliefs = beliefs.to_owned();
            for opp_hand_idx in 0..beliefs.len() {
                if beliefs[opp_hand_idx] == 0f64 {
                    continue;
                }
                let opp_hole_cards =
                    self.hand_ranges[usize::from(1 - br_player)].hands[opp_hand_idx];
                let opp_bucket = self.get_bucket(opp_hole_cards, &next_state);
                let is_key = format!("{}{}", opp_bucket, next_state.history_string());
                let opp_strategy = self
                    .infosets
                    .get_or_insert(is_key, opp_action_count)
                    .current_strategy();
                fp += beliefs[opp_hand_idx] * opp_strategy[CHECK_FOLD_IDX];
                new_beliefs[opp_hand_idx] *= 1f64 - opp_strategy[CHECK_FOLD_IDX];
            }
            normalize(&mut new_beliefs);
            let wp = self.wp_rollout(next_state.clone(), &new_beliefs, br_player);
            action_utils[i] =
                (fp * pot) + ((1f64 - fp) * (wp * br_amt)) - ((1f64 - wp) * (asked + br_amt));
        }
        // get action that has maximum utility
        let mut max_util = 0f64;
        let mut max_action_idx = 0;
        for (i, util) in action_utils.iter().enumerate() {
            if *util > max_util {
                max_util = *util;
                max_action_idx = i;
            }
        }
        if max_util > 0f64 {
            return valid_actions[max_action_idx];
        }
        valid_actions[CHECK_FOLD_IDX]
    }
    /// Calcuates win percentage if both players check/call from this node onward
    /// Exhaustivly deals all possible remaining board cards and computes the mean probability
    /// of winning with hand h with opponents range
    fn wp_rollout(&self, node: GameState, beliefs: &Vec<f64>, br_player: u8) -> f64 {
        let mut wins = 0f64;
        let mut games = 0f64;
        // copy board cards
        let mut board_card_count = 0;
        let mut used_cards_mask = 0u64;
        let mut board: Hand = Hand::default();
        for (i, card) in node.board().iter().enumerate() {
            if *card >= 52 {
                break;
            }
            board += CARDS[usize::from(*card)];
            board_card_count += 1;
            used_cards_mask |= 1u64 << *card;
        }
        let our_cards = node.player(br_player.into()).cards();
        let our_hand: Hand = CARDS[usize::from(our_cards[0])] + CARDS[usize::from(our_cards[1])];
        used_cards_mask |= (1u64 << our_cards[0]) | (1u64 << our_cards[1]);
        // iterate over all possible boards
        if 5 - board_card_count == 0 {
            let our_score = evaluate(&(our_hand + board));
            for (i, prob) in beliefs.iter().enumerate() {
                if *prob == 0f64 {
                    continue;
                }
                let opp_combo = &self.hand_ranges[usize::from(1 - br_player)].hands[i];
                let opp_combo_mask = (1u64 << opp_combo.0) | (1u64 << opp_combo.1);
                if (opp_combo_mask & used_cards_mask) != 0 {
                    continue;
                }
                let opp_hand: Hand =
                    CARDS[usize::from(opp_combo.0)] + CARDS[usize::from(opp_combo.1)];
                let opp_score = evaluate(&(opp_hand + board));
                if our_score > opp_score {
                    wins += *prob;
                }
                if our_score == opp_score {
                    wins += *prob * 0.5;
                }
                games += *prob;
            }
            return wins / games;
        }
        let board_combos = CardComboIter::new(used_cards_mask, 5 - board_card_count);
        for combo in board_combos {
            // get eval combo
            let mut board_combo = board;
            let mut used_cards_mask = used_cards_mask;
            for card in combo.iter() {
                board_combo += CARDS[usize::from(*card)];
                used_cards_mask |= 1u64 << *card;
            }
            let our_score = evaluate(&(our_hand + board_combo));
            for (i, prob) in beliefs.iter().enumerate() {
                if *prob == 0f64 {
                    continue;
                }
                let opp_combo = &self.hand_ranges[usize::from(1 - br_player)].hands[i];
                let opp_combo_mask = (1u64 << opp_combo.0) | (1u64 << opp_combo.1);
                if (opp_combo_mask & used_cards_mask) != 0 {
                    continue;
                }
                let opp_hand: Hand =
                    CARDS[usize::from(opp_combo.0)] + CARDS[usize::from(opp_combo.1)];
                let opp_score = evaluate(&(opp_hand + board_combo));
                if our_score > opp_score {
                    wins += *prob;
                }
                if our_score == opp_score {
                    wins += *prob * 0.5;
                }
                games += *prob;
            }
        }
        wins / games
    }
    fn get_bucket(&self, hole_cards: Combo, node: &GameState) -> u32 {
        let mut cards: [Card; 7] = [52; 7];
        cards[0] = hole_cards.0;
        cards[1] = hole_cards.1;
        for i in 0..5 {
            cards[i + 2] = node.board()[i];
        }
        let hand_index = self.hand_indexers[usize::from(node.round())].get_index(&cards);
        // let bucket = self.card_abstraction[usize::from(node.round())].get(hand_index as usize);
        hand_index as u32
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
