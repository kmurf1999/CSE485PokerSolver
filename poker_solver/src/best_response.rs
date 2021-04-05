use crate::action::{Action, CHECK_CALL_IDX, FOLD_IDX};
use crate::combos::CardComboIter;
use crate::infoset::sample_action_index;
use crate::normalize;
use crate::solver::Solver;
use crate::state::GameState;
use rand::prelude::SmallRng;
use rust_poker::hand_evaluator::{evaluate, Hand, CARDS};

use rand::Rng;
use rand::SeedableRng;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

pub struct BrThread<'a> {
    solver: &'a Solver,
    strategy_cache: HashMap<String, Vec<f64>>,
    counter: Arc<AtomicUsize>,
    rng: SmallRng,
}

impl<'a> BrThread<'a> {
    fn new(solver: &'a Solver, counter: Arc<AtomicUsize>) -> Self {
        BrThread {
            solver,
            strategy_cache: HashMap::default(),
            counter,
            rng: SmallRng::from_entropy(),
        }
    }

    fn run(&mut self, max_iter: usize) -> [f64; 2] {
        let mut equity = [0f64; 2];

        while self.counter.fetch_add(1, Ordering::SeqCst) < max_iter {
            // setup beliefs for both hand ranges
            let mut beliefs: Vec<Vec<f64>> = self
                .solver
                .hand_ranges
                .iter()
                .map(|hr| {
                    hr.hands
                        .iter()
                        .map(|combo| combo.2 as f64)
                        .collect::<Vec<f64>>()
                })
                .collect();
            // create initial state and sample private chance
            let deal = self
                .solver
                .initial_state
                .sample_private_chance_from_ranges(&mut self.rng, &self.solver.hand_ranges);
            let initial_state = self.solver.initial_state.apply_action(deal);
            // update beliefs using private chance cards
            for (player, player_beliefs) in beliefs.iter_mut().enumerate() {
                let other_player_cards = initial_state.player(1 - player).cards();
                let other_player_card_mask =
                    (1u64 << other_player_cards[0]) | (1u64 << other_player_cards[1]);
                for (i, combo) in self.solver.hand_ranges[player].hands.iter().enumerate() {
                    let combo_mask = (1u64 << combo.0) | (1u64 << combo.1);
                    if (combo_mask & other_player_card_mask) != 0 {
                        player_beliefs[i] = 0f64;
                    }
                }
                normalize(player_beliefs);
            }
            // do local br
            for br_player in 0..2 {
                equity[usize::from(br_player)] += self.traverse_br(
                    initial_state.clone(),
                    &mut beliefs[usize::from(1 - br_player)],
                    br_player,
                );
            }
        }
        equity
        // for _ in 0..iterations {
        //     let mut beliefs: Vec<Vec<f64>> = self
        //         .hand_ranges
        //         .iter()
        //         .map(|hr| {
        //             hr.hands
        //                 .iter()
        //                 .map(|combo| combo.2 as f64)
        //                 .collect::<Vec<f64>>()
        //         })
        //         .collect();
        //     let deal = self
        //         .initial_state
        //         .sample_private_chance_from_ranges(&mut self.rng, &self.hand_ranges);
        //     let initial_state = self.initial_state.apply_action(deal);
        //     for (player, player_beliefs) in beliefs.iter_mut().enumerate() {
        //         let other_player_cards = initial_state.player(1 - player).cards();
        //         let other_player_card_mask =
        //             (1u64 << other_player_cards[0]) | (1u64 << other_player_cards[1]);
        //         for (i, combo) in self.hand_ranges[player].hands.iter().enumerate() {
        //             let combo_mask = (1u64 << combo.0) | (1u64 << combo.1);
        //             if (combo_mask & other_player_card_mask) != 0 {
        //                 player_beliefs[i] = 0f64;
        //             }
        //         }
        //         normalize(player_beliefs);
        //     }
        //     for br_player in 0..2 {
        //         equity[usize::from(br_player)] += self.traverse_br(
        //             initial_state.clone(),
        //             &mut beliefs[usize::from(1 - br_player)],
        //             br_player,
        //             &mut opp_strategy_cache,
        //             &mut rng,
        //         );
        //     }
        // }
    }
    fn traverse_br(&mut self, node: GameState, beliefs: &mut Vec<f64>, br_player: u8) -> f64 {
        let solver = self.solver;
        // return the reward for this player
        if node.is_terminal() {
            return node.player_reward(usize::from(br_player));
        }
        // sample a single chance outcome
        if node.is_chance() {
            let action: Action;
            if node.is_initial_state() {
                panic!("should deal in run_br");
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
                    for (i, combo) in solver.hand_ranges[usize::from(1 - br_player)]
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
            normalize(beliefs);
            return self.traverse_br(node.apply_action(action), beliefs, br_player);
        }
        if node.current_player() == br_player as i8 {
            // calculate action using br
            let action = self.local_br(node.clone(), beliefs, br_player);
            self.traverse_br(node.apply_action(action), beliefs, br_player)
        } else {
            // calculate action using cummulative strategy
            let legal_actions = node.valid_actions(&solver.betting_abstraction);
            let action_count = legal_actions.len();
            let hole_cards = node.acting_player().cards();
            let hand_bucket = solver.get_bucket(hole_cards[0], hole_cards[1], &node);
            let is_key = format!("{}{}", hand_bucket, node.history_string());
            let strategy =
                self.strategy_cache
                    .entry(is_key.clone())
                    .or_insert_with(|| match solver.infosets.get(is_key) {
                        Some(val) => val.average_strategy(),
                        None => vec![1.0 / action_count as f64; action_count],
                    });
            let a_idx = sample_action_index(&strategy, &mut self.rng);
            for (i, hand) in solver.hand_ranges[usize::from(1 - br_player)]
                .hands
                .iter()
                .enumerate()
            {
                if beliefs[i] == 0f64 {
                    continue;
                }
                let hand_bucket = solver.get_bucket(hand.0, hand.1, &node);
                let is_key = format!("{}{}", hand_bucket, node.history_string());
                let strategy = self
                    .strategy_cache
                    .entry(is_key.clone())
                    .or_insert_with(|| match solver.infosets.get(is_key) {
                        Some(val) => val.average_strategy(),
                        None => vec![1.0 / action_count as f64; action_count],
                    });
                beliefs[i] *= strategy[a_idx];
            }
            normalize(beliefs);
            self.traverse_br(node.apply_action(legal_actions[a_idx]), beliefs, br_player)
        }
    }
    /// Calculates best response action using local br function
    fn local_br(&mut self, node: GameState, beliefs: &[f64], br_player: u8) -> Action {
        let solver = self.solver;
        let valid_actions = node.valid_actions(&self.solver.betting_abstraction);
        let mut action_utils = vec![0f64; valid_actions.len()];
        // get check/call util
        let wp = self.wp_rollout(node.clone(), &beliefs, br_player);
        let pot = node.pot() as f64;
        let asked = (node.player(usize::from(1 - br_player)).wager()
            - node.player(usize::from(br_player)).wager()) as f64;
        // the utility for call
        action_utils[CHECK_CALL_IDX] = (wp * pot) - ((1f64 - wp) * asked);
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
            let opp_legal_actions = next_state.valid_actions(&solver.betting_abstraction);
            let opp_action_count = opp_legal_actions.len();
            // loop over opponent range
            let mut new_beliefs = beliefs.to_vec();
            for opp_hand_idx in 0..beliefs.len() {
                if beliefs[opp_hand_idx] == 0f64 {
                    continue;
                }
                let opp_hole_cards =
                    solver.hand_ranges[usize::from(1 - br_player)].hands[opp_hand_idx];
                let opp_bucket = solver.get_bucket(opp_hole_cards.0, opp_hole_cards.1, &next_state);
                let is_key = format!("{}{}", opp_bucket, next_state.history_string());
                let opp_strategy = self
                    .strategy_cache
                    .entry(is_key.clone())
                    .or_insert_with(|| match solver.infosets.get(is_key) {
                        Some(val) => val.average_strategy(),
                        None => vec![1.0 / opp_action_count as f64; opp_action_count],
                    });
                fp += beliefs[opp_hand_idx] * opp_strategy[FOLD_IDX];
                new_beliefs[opp_hand_idx] *= 1f64 - opp_strategy[FOLD_IDX];
            }
            normalize(&mut new_beliefs);
            let wp = self.wp_rollout(next_state.clone(), &new_beliefs, br_player);
            action_utils[i] =
                fp * pot + (1.0 - fp) * (wp * (pot + br_amt) - (1.0 - wp) * (asked + br_amt));
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
        valid_actions[FOLD_IDX]
    }
    /// Calcuates win percentage if both players check/call from this node onward
    /// Exhaustivly deals all possible remaining board cards and computes the mean probability
    /// of winning with hand h with opponents range
    pub fn wp_rollout(&mut self, node: GameState, beliefs: &[f64], br_player: u8) -> f64 {
        let mut wins = 0f64;
        let mut games = 0f64;
        // copy board cards
        let mut board_card_count = 0;
        let mut used_cards_mask = 0u64;
        let mut board: Hand = Hand::default();
        for card in node.board() {
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

        if 5 - board_card_count == 0 {
            let our_score = evaluate(&(our_hand + board));
            for (i, prob) in beliefs.iter().enumerate() {
                if *prob == 0f64 {
                    continue;
                }
                let opp_combo = &self.solver.hand_ranges[usize::from(1 - br_player)].hands[i];
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
        } else {
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
                    let opp_combo = &self.solver.hand_ranges[usize::from(1 - br_player)].hands[i];
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
        }
        wins / games
    }
}

pub fn run_local_br(solver: &Solver, iterations: usize) -> [f64; 2] {
    let equity = Arc::new(Mutex::new([0f64; 2]));
    let counter = Arc::new(AtomicUsize::new(0));
    const N_THREADS: usize = 8;
    crossbeam::scope(|scope| {
        for _ in 0..N_THREADS {
            let equity = equity.clone();
            let counter = counter.clone();
            scope.spawn(move |_| {
                let mut br_thread = BrThread::new(solver, counter.clone());
                let thread_equity = br_thread.run(iterations);
                let mut equity = equity.lock().unwrap();
                for i in 0..2 {
                    equity[i] += thread_equity[i];
                }
            });
        }
    })
    .unwrap();
    let mut equity = Arc::try_unwrap(equity).unwrap().into_inner().unwrap();
    for i in 0..2 {
        equity[i] /= 0.5 * iterations as f64;
    }

    equity
}
