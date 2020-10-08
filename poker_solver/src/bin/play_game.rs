use poker_solver::agents::{Agent, HumanAgent, RandomAgent};
use rand::thread_rng;
use rand::rngs::ThreadRng;
use std::iter::FromIterator;
use poker_solver::state::{GameState, BettingRound};

use rust_poker::hand_evaluator::{evaluate, CARDS, Hand};
use rust_poker::hand_range::Combo;
use rust_poker::constants::{ RANK_TO_CHAR, SUIT_TO_CHAR };

/// Scores the hand
/// 
/// Makes the best 5 card hand from 7 cards and generates a score
/// higher score is better
fn score_hand(board: &[u8], private_cards: &[u8]) -> u16 {
    let mut hand = Hand::empty();
    board.into_iter().for_each(|c| {
        hand += CARDS[usize::from(*c)];
    });
    private_cards.into_iter().for_each(|c| {
        hand += CARDS[usize::from(*c)];
    });
    return evaluate(&hand);
}

/// Transform card array into a readable string
fn cards_to_str(cards: &[u8]) -> String {
    let mut chars: Vec<char> = Vec::new();
    cards.into_iter().for_each(|c| {
        chars.push(RANK_TO_CHAR[usize::from(*c >> 2)]);
        chars.push(SUIT_TO_CHAR[usize::from(*c & 3)]);
    });
    return String::from_iter(chars);
}

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
    pub fn play(&mut self) {
        println!("GAME START");
        // while game isn't over
        while !self.game_is_over() {
            // simulate a single hand
            println!("");
            println!("Dealing new hand...");
            println!("Current stack sizes [{} : {}]", self.stacks[0], self.stacks[1]);
            println!("");

            // create a state object using current stacks as initial stacks
            let mut state = GameState::new(10, self.stacks.clone());
            // deal cards to both players
            state.deal_cards(&mut self.rng);

            // get next action self.agents[state.current_player].get_action
            while !state.is_game_over() {
                let acting_player = usize::from(state.get_current_player_idx());
                let action = self.agents[acting_player].get_action(&state);
                // print to terminal
                println!("Player {} has chosen to {}", acting_player, action);
                state = state.apply_action(&mut self.rng, action);
            }

            // final pot value
            let pot = state.get_pot();
            // copy stack values after hand
            // (before we award chips)
            self.stacks[0] = state.get_player(0).get_stack();
            self.stacks[1] = state.get_player(1).get_stack();

            println!("");
            println!("Hand has ended");
            println!("");

            if let Some(player_fold) = state.player_folded() {
                println!("Player {} has folded.  Player {} wins {}", player_fold, 1 - player_fold, pot);
                // handle fold
                // award chips to winner
                self.stacks[1 - usize::from(player_fold)] += pot;
            } else {
                println!("The board is [{}]", cards_to_str(state.get_board()));
                println!("Player {} has [{}]", 0, cards_to_str(state.get_player(0).get_cards()));
                println!("Player {} has [{}]", 1, cards_to_str(state.get_player(1).get_cards()));
                // deal cards until there are 5
                while state.get_round() != BettingRound::RIVER {
                    // next round deals cards and increments round
                    state.next_round(&mut self.rng);
                }
                // evaluate winner
                // create public cards
                let board = state.get_board();
                let player_0_score = score_hand(board, state.get_player(0).get_cards());
                let player_1_score = score_hand(board, state.get_player(1).get_cards());
                println!("{} {}", player_0_score, player_1_score);
                if player_0_score == player_1_score {
                    println!("Tie!");
                    // tie
                    self.stacks[0] += pot / 2;
                    self.stacks[1] += pot / 2;
                } else if player_0_score > player_1_score {
                    println!("Player 0 wins {}", pot);
                    // player 0 win
                    self.stacks[0] += pot;
                } else {
                    println!("Player 1 wins {}", pot);
                    // player 1 win
                    self.stacks[1] += pot;
                }
            }

        }
    }
    /// Return true is game has finished
    /// 
    /// This checks to see if both players have money
    fn game_is_over(&self) -> bool {
        return !(self.stacks[0] > 0 && self.stacks[1] > 0);
    }
}

fn main() {
    let mut game = GameEnvironment {
        agents: [
            Box::new(RandomAgent::new()),
            Box::new(RandomAgent::new())
        ],
        rng: thread_rng(),
        stacks: [10000; 2]
    };
    game.play();
}