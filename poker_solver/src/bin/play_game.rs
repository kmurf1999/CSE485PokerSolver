use poker_solver::agents::{Agent, HumanAgent, RandomAgent};
use rand::thread_rng;
use rand::rngs::ThreadRng;

/// Simulates HUNL Texas Holdem game between two agents
pub struct GameEnvironment {
    /// Represents the two players
    agents: [Box<dyn Agent>; 2],
    /// update stack size for each player after every hand
    stacks: [u32; 2],
    /// A seeded rng object for generating random numbers
    rng: ThreadRng
}

impl GameEnvironment {
    /// Starts and runs poker game until completion
    pub fn play(&self) {
        // while game isn't over
        while true {
            // simulate a single hand
            // create a state object using current stacks as initial stacks
            // get next action self.agents[state.current_player].get_action
            // apply action to state
            // continue until hand is over
            // once hand is over evaluate the winner and update stack sizes
            break;
        }
    }
}

fn main() {
    let mut game = GameEnvironment {
        agents: [
            Box::new(HumanAgent::new()),
            Box::new(RandomAgent::new())
        ],
        rng: thread_rng(),
        stacks: [10000; 2]
    };
    game.play();
}