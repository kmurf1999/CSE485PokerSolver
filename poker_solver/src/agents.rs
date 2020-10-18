use crate::state::{Action, GameState};
use rand::thread_rng;
use rand::Rng;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use std::io::{self, BufRead};

use crate::card::{cards_to_str, player_hand_score};
use colored::Colorize;

/// Entity that employs a strategy to play poker
/// 
/// An agent subscribes to a poker game environment 
/// and chooses from available actions until the game is done
pub trait Agent {
    /// Selects a valid action to play
    fn get_action(&mut self, state: &GameState) -> Action;
}

/// RandomAgent is an agent with a random strategy profile
/// 
/// It selects from available actions at random
#[derive(Debug)]
pub struct RandomAgent {
    rng: ThreadRng
}

impl Agent for RandomAgent {
    /// Get random valid action
    fn get_action(&mut self, state: &GameState) -> Action {
        // If action is BET or RAISE it ensures that
        // the amount is also valid
        let actions = state.valid_actions();
        let chosen_action = actions.choose(&mut self.rng)
            .unwrap()
            .to_owned();

        if let Action::BET(_) = chosen_action {
            let max_bet = state.current_player().stack();
            let bet_size = self.rng.gen_range(1, max_bet);
            return Action::BET(bet_size);
        }

        if let Action::RAISE(_) = chosen_action {
            let max_raise = state.current_player().stack() - (state.other_player().wager() - state.current_player().wager());
            let raise_size = self.rng.gen_range(1, max_raise);
            return Action::RAISE(raise_size);
        }

        return chosen_action;
    }
}

impl RandomAgent {
    pub fn new() -> RandomAgent {
        RandomAgent {
            rng: thread_rng()
        }
    }
}

/// Human is an Agent that is controlled by a human
/// 
/// It takes inputs from STDIN
/// retrys until input is valid
#[derive(Debug)]
pub struct HumanAgent {

}

impl Agent for HumanAgent {
    /// Get valid action from STDIN
    fn get_action(&mut self, state: &GameState) -> Action {
        // make sure that if chosen action is bet or raise,
        // the the bet or raise size makes sense and is valid
        let actions = state.valid_actions();
        let stdin = io::stdin();
        let mut is_action_valid = false;

        println!("");
        println!("{}", "Please select an action.".bright_cyan());
        println!("{} {} {} {}", "You have".bright_yellow(), state.current_player().stack().to_string().red(), "chips, your opponent has".bright_yellow(), state.other_player().stack().to_string().red());
        println!("The pot is: {}", state.pot().to_string().green());
        println!("The board cards are: [{}]", cards_to_str(state.board()).to_string().bright_yellow());
        println!("{} cards are: [{}]", "Your".red(), cards_to_str(state.current_player().cards()).to_string().bright_yellow());

        //Hand Strength -> Human play
        //Suggestion to Play this hand or not.
        //Will be used for The RandomAgent brain in the future
        let player_strength = player_hand_score(state.current_player().cards());




        if player_strength >= 4399 {
            println!("{} ", "You have strong cards, Call it !".red())
        } else {
            println!("{} ", "You have weak cards, there might be a risk !".red())
        }

        //if state.round() == BettingRound::FLOP {
        //    println!("Board strength: {:?}", board_strength.to_string());
        //}

        while !is_action_valid {
            is_action_valid = true;
            // List Valid actions
            actions.iter().enumerate().for_each(|(i, a)| {
                match a {
                    Action::BET(_) => println!("{}: {}", i.to_string().red(), "Bet".bright_cyan()),
                    Action::RAISE(_) => println!("{}: {}", i.to_string().red(), "Raise".bright_cyan()),
                    Action::CALL => {
                        let call_amt = state.other_player().wager() - state.current_player().wager();
                        println!("{}: {} {}", i.to_string().red(), "Call".bright_cyan(),call_amt.to_string().bright_yellow());
                    },
                    Action::FOLD => println!("{}: {}", i.to_string().red(), "Fold".bright_cyan()),
                    Action::CHECK => println!("{}: {}", i.to_string().red(), "Check".bright_cyan()),
                }
            });
            // get input
            let mut input = String::new();
            stdin.lock().read_line(&mut input).expect("could not read line");
            // ensure input is a number in correct range
            let action_index = match input.trim().parse::<usize>() {
                Ok(num) => {
                    if num > actions.len() - 1 {
                        is_action_valid = false;
                        println!("Action must be between {} and {}", 0, actions.len() - 1);
                        continue;
                    }
                    num
                },
                Err(_) => {
                    is_action_valid = false;
                    println!("Input must be a number. Retrying.");
                    continue;
                }
            };
            // ensure bet is correct size
            if let Action::BET(_) = actions[action_index] {
                let max_bet = state.current_player().stack();
                let mut bet_s = String::new();
                println!("Please input a bet size from ({}, {}): ", 1, max_bet);
                stdin.lock().read_line(&mut bet_s).unwrap();
                match bet_s.trim().parse::<u32>() {
                    Ok(num) => {
                        if num == 0 || num > max_bet {
                            is_action_valid = false;
                            println!("Bet size out of range.  Retrying.");
                            continue;
                        }
                        if is_action_valid {
                            return Action::BET(num);
                        }
                    },
                    Err(_) => {
                        is_action_valid = false;
                        println!("Bet size must be a floating point number. Retrying.");
                        continue;
                    }
                };
            }
            if let Action::RAISE(_) = actions[action_index] {
                let max_raise = state.current_player().stack() - (state.other_player().wager() - state.current_player().wager());
                let mut raise_s = String::new();
                println!("Please input a raise size from ({},{}) over opponent raise: ", 1, max_raise);
                stdin.lock().read_line(&mut raise_s).unwrap();
                match raise_s.trim().parse::<u32>() {
                    Ok(num) => {
                        if num == 1 || num > max_raise {
                            is_action_valid = false;
                            println!("Raise size out of range.  Retrying.");
                            continue;
                        }
                        if is_action_valid {
                            return Action::RAISE(num);
                        }
                    },
                    Err(_) => {
                        is_action_valid = false;
                        println!("Raise size must be a number greater than 0. Retrying.");
                        continue;
                    }
                };
            }
            // if CALL, CHECK, or FOLD return
            if is_action_valid {
                return actions[action_index];
            }
        }
        // should never reach this
        return actions[0];
    }
}

impl HumanAgent {
    pub fn new() -> HumanAgent {
        HumanAgent {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
