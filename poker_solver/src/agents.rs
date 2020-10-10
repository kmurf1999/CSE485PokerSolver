use crate::state::{ Action, GameState };
use rand::thread_rng;
use rand::Rng;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;

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
            // Stack to pot ratio
            let spr = state.current_player().get_stack() as f32 / state.get_pot() as f32;
            let bet_size = self.rng.gen_range(0.0, spr);
            return Action::BET(bet_size);
        }

        if let Action::RAISE(_) = chosen_action {
            let swr = state.current_player().get_stack() as f32 / state.other_player().get_wager() as f32;
            let raise_size = self.rng.gen_range(1.0, swr);
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

        //List Valid actions
        eprintln!("Valid actions: {:?}", actions);
        println!("Please select an action: ");
        //getting user input
        let mut input= String::new();
        let mut chosen_action:Action = Action::FOLD;

        io::stdin().read_line(&mut input);

                //CALL
                if input.contains("CALL"){

                    chosen_action = Action::CALL;

                    //return Action::CALL;

                //FOLD
                } else if input.contains("FOLD"){

                    chosen_action = Action::FOLD;
                    //return Action::FOLD;

                //BET
                } else if input.contains("BET"){

                    let spr = state.current_player().get_stack() as f32 / state.get_pot() as f32;
                    let mut bet_s = String::new();
                    println!("Please input bet size (Range {},{}): ", 0.0, spr);
                    io::stdin().read_line(&mut bet_s);
                    let bet_value = bet_s.trim().parse::<f32>().unwrap();

                    if bet_value < 0.0 && bet_value > spr {
                        println!("Out of range !");
                        return Action::FOLD;
                    } else {
                        chosen_action = Action::BET(bet_value);
                        //return Action::BET(bet_value);
                    }


                } else if input.contains("RAISE"){

                    let swr = state.current_player().get_stack() as f32 / state.other_player().get_wager() as f32;

                    let mut raise_size = String::new();
                    println!("Please input bet size (Range {},{}): ", 1.0, swr);
                    io::stdin().read_line(&mut raise_size);
                    let raise_value = raise_size.trim().parse::<f32>().unwrap();

                    if raise_value < 1.0 && raise_value > swr {
                        println!("Out of range !");

                        return Action::FOLD;
                    } else {

                        chosen_action = Action::RAISE(raise_value)
                        //return Action::BET(raise_value);
                    }

                } else if input.contains("CHECK"){

                    chosen_action = Action::CHECK;
                    //return Action::CHECK;
                }




        return chosen_action;

        //unimplemented!();
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
