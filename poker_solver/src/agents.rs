use crate::state::{ Action, GameState };
use rand::thread_rng;
use rand::rngs::ThreadRng;

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
        let valid_actions = state.valid_actions()
        let chosen_action = valid_actions.choose(&mut self.rng)
        
        if chosen_action::BET{
            
            //check if current stack can bet.
            //Also if agent folded we need to make sure he wont play BET action again.
            //this check needs FIX, im not sure what to call here ( we should get the current_stack & make sure agent not folded )
            // if current_stack >= 0 && !folded {
            // if current_stack == 0 { return EMPTY;} ~~~~> wait for next game.
            
            //getting random bet size
            // stack_size / pot_size
            let stack_ov_pot: f64 = stack_size/pot_size;
            //I will make it start from 1 bet, because i guess we cant bet 0. correct if wrong
            let rand_bet = rng.gen_range(1.0, stack_ov_pot);
            return Action::BET;
            
            
            //} end check stack_size if
            //return FOLDED;
            
        }
        
        if chosen_action::RAISE{
            
            //check if current stack can RAISE.
            //Also if agent folded we need to make sure he wont play RAISE action again.
            //If another agent raised
            // if ... {
            
            
            
            
            return Action::RAISE;
            
            //} end raise conditions if
            //return FOLDED;
        }
        
        return Action::FOLD;
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
