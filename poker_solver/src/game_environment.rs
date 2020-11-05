// use rand::thread_rng;
use rand::rngs::ThreadRng;
use crate::agents::Agent;
use crate::state::GameState;
use crate::round::BettingRound;
use crate::card::{cards_to_str, score_hand};
use colored::*;

/// Simulates HUNL Texas Holdem game between two agents
pub struct GameEnvironment {
    /// Represents the two players
    pub agents: [Box<dyn Agent>; 2],
    /// update stack size for each player after every hand
    pub stacks: [u32; 2],
    /// A seeded rng object for generating random numbers
    pub rng: ThreadRng
}

impl GameEnvironment {
    /// Starts and runs poker game until completion
    pub fn play(&mut self) {
        // while game isn't over
        while !self.game_is_over() {
            // simulate a single hand
            println!("");
            println!("{}","Dealing new hand".bright_red());
            println!("Current stack sizes [{} : {}]", self.stacks[0], self.stacks[1]);
            println!("");

            println!("Posting blinds & Dealing cards");
            println!("");
            // big blind is 10
            // small blind is 5

            // create a state object using current stacks as initial stacks
            let mut state = GameState::init_with_blinds(self.stacks, [10, 5], None);
            // deal cards to both players
            state.deal_cards(&mut self.rng);

            while !state.is_game_over() {
                let acting_player = usize::from(state.current_player_idx());
                let action = self.agents[acting_player].get_action(&state);
                // print to terminal
                println!("Player {} has chosen to {}", acting_player, action);
                state = state.apply_action(action);
                println!("Stacks: [{}, {}],  Pot: {}", state.player(0).stack(), state.player(1).stack(), state.pot());
                println!("");
                // if betting is finished, advance to next round
                if state.bets_settled() && !state.is_game_over() {
                    state = state.next_round();
                    state.deal_cards(&mut self.rng);
                    println!("Dealing {}...", state.round());
                    println!("Board cards: [{}]", cards_to_str(state.board()));
                }
            }

            // final pot value
            let pot = state.pot();
            // copy stack values after hand
            // (before we award chips)
            self.stacks[0] = state.player(0).stack();
            self.stacks[1] = state.player(1).stack();

            if let Some(player_fold) = state.player_folded() {
                println!("Player {} has folded.  Player {} wins {}", player_fold, 1 - player_fold, pot);
                // handle fold
                // award chips to winner
                self.stacks[1 - usize::from(player_fold)] += pot;
            } else {
                // deal cards until there are 5
                while state.round() != BettingRound::RIVER {
                    // next round deals cards and increments round
                    state = state.next_round();
                    state.deal_cards(&mut self.rng);
                }
                println!("The board is [{}]", cards_to_str(state.board()));
                println!("Player {} has [{}]", 0, cards_to_str(state.player(0).cards()));
                println!("Player {} has [{}]", 1, cards_to_str(state.player(1).cards()));

                // evaluate winner
                // create public cards
                let board = state.board();
                let player_0_score = score_hand(board, state.player(0).cards());
                let player_1_score = score_hand(board, state.player(1).cards());

                // println!("Player 0 scored: {}", player_0_score);
                // println!("Player 1 scored: {}", player_1_score);
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

            // So each hand players take turn going first
            self.stacks.reverse();
            self.agents.reverse();
        }

        println!("Game over");
    }
    /// Return true is game has finished
    /// 
    /// This checks to see if both players have money
    fn game_is_over(&self) -> bool {
        return !(self.stacks[0] > 0 && self.stacks[1] > 0);
    }
}