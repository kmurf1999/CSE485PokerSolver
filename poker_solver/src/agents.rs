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
        unimplemented!();
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