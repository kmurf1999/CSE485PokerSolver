use crate::action::Action;
use crate::card::cards_to_str;
use crate::codec::{PokerCodec, PokerCodecError};
use crate::event::{PokerEvent, PokerEventType};
use crate::round::BettingRound;
use crate::state::GameState;
use colored::Colorize;

use futures::SinkExt;
use rand::prelude::SliceRandom;
use rand::{thread_rng, Rng};
use std::time::Duration;

use std::error::Error;
use std::io::{self, BufRead};
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::net::TcpStream;
use tokio::stream::{Stream, StreamExt};
use tokio_util::codec::Framed;

pub struct RandomAgent {
    addr: SocketAddr,
    codec: Framed<TcpStream, PokerCodec>,
    position: u8,
    hand_state: GameState,
}

impl RandomAgent {
    pub async fn start(server_addr: SocketAddr) -> Result<(), Box<dyn Error>> {
        let stream = TcpStream::connect(server_addr).await?;
        let addr = stream.local_addr()?;
        let codec = Framed::new(stream, PokerCodec::new());
        let mut agent = RandomAgent {
            addr,
            codec,
            position: 0,
            hand_state: GameState::default(),
        };
        agent.play().await?;
        Ok(())
    }
    async fn play(&mut self) -> Result<(), Box<dyn Error>> {
        while let Some(msg) = self.next().await {
            match msg {
                Ok(event) => {
                    self.handle_event(event).await?;
                }
                Err(err) => {
                    println!("client recv error: {}", err);
                }
            }
        }
        Ok(())
    }
    async fn handle_event(&mut self, event: PokerEvent) -> Result<(), Box<dyn Error>> {
        match event.event {
            PokerEventType::HandStart {
                stacks,
                blinds,
                position,
            } => {
                self.hand_state = GameState::init_with_blinds(stacks, blinds, None);
                self.position = position;
            }
            PokerEventType::HandOver {
                winner,
                stacks,
                pot,
            } => {
                // idk do something
            }
            PokerEventType::DealCards {
                round,
                n_cards,
                cards,
            } => {
                if round != BettingRound::PREFLOP {
                    self.hand_state = self.hand_state.next_round();
                }
                // do something
            }
            PokerEventType::RequestAction => {
                // apply valid action
                let action = self.random_action();
                // send action to server
                tokio::time::sleep(Duration::from_secs(3)).await;
                self.codec
                    .send(PokerEvent {
                        from: self.addr,
                        event: PokerEventType::SendAction { action },
                    })
                    .await?;
            }
            PokerEventType::AlertAction {
                action,
                player,
                pot,
                stacks,
                wagers,
            } => {
                self.hand_state = self.hand_state.apply_action(action);
            }
            _ => {}
        }
        Ok(())
    }
    fn random_action(&mut self) -> Action {
        // If action is BET or RAISE it ensures that
        // the amount is also valid
        let mut rng = thread_rng();
        let actions = self.hand_state.valid_actions();
        let chosen_action = actions.choose(&mut rng).unwrap().to_owned();

        if let Action::BET(_) = chosen_action {
            let min_bet = std::cmp::min(10, self.hand_state.current_player().stack());
            let max_bet = self.hand_state.current_player().stack();
            let bet_size = rng.gen_range(min_bet, max_bet);
            return Action::BET(bet_size);
        }

        if let Action::RAISE(_) = chosen_action {
            let min_raise = 2 * self.hand_state.other_player().wager();
            if self.hand_state.current_player().stack() < min_raise {
                return Action::RAISE(self.hand_state.current_player().stack());
            } else {
                let max_raise = self.hand_state.current_player().stack();
                let raise_size = rng.gen_range(min_raise, max_raise);
                return Action::RAISE(raise_size);
            }
        }

        return chosen_action;
    }
}

impl Stream for RandomAgent {
    type Item = Result<PokerEvent, PokerCodecError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Secondly poll the `Framed` stream.
        let result: Option<_> = futures::ready!(Pin::new(&mut self.codec).poll_next(cx));

        Poll::Ready(result)
    }
}

pub struct HumanAgent {
    addr: SocketAddr,
    codec: Framed<TcpStream, PokerCodec>,
    position: u8,
    hand_state: GameState,
}

impl HumanAgent {
    pub async fn start(server_addr: SocketAddr) -> Result<(), Box<dyn Error>> {
        let stream = TcpStream::connect(server_addr).await?;
        let addr = stream.local_addr()?;
        let codec = Framed::new(stream, PokerCodec::new());
        let mut agent = HumanAgent {
            addr,
            codec,
            position: 0,
            hand_state: GameState::default(),
        };
        agent.play().await?;
        Ok(())
    }
    async fn play(&mut self) -> Result<(), Box<dyn Error>> {
        while let Some(msg) = self.next().await {
            match msg {
                Ok(event) => {
                    self.handle_event(event).await?;
                }
                Err(err) => {
                    println!("client recv error: {}", err);
                }
            }
        }
        Ok(())
    }
    async fn handle_event(&mut self, event: PokerEvent) -> Result<(), Box<dyn Error>> {
        match event.event {
            PokerEventType::HandStart {
                stacks,
                blinds,
                position,
            } => {
                self.hand_state = GameState::init_with_blinds(stacks, blinds, None);
                self.position = position;
                println!("---- Hand Starting ----");
                println!("Blinds: {}, {}", blinds[0], blinds[1]);
                println!(
                    "Hero stack: {} | Villan Stack: {}",
                    stacks[usize::from(position)],
                    stacks[usize::from(1 - position)]
                );
            }
            PokerEventType::HandOver {
                winner,
                stacks,
                pot,
            } => {
                println!("---- Hand Over ----");
                match (winner, winner == self.position) {
                    (2, _) => println!("Tie!"),
                    (_, true) => println!("You win!"),
                    (_, false) => println!("You lose"),
                };
                println!("Pot: {}", pot);
                println!(
                    "Hero stack: {} | Villan Stack: {}",
                    stacks[usize::from(self.position)],
                    stacks[usize::from(1 - self.position)]
                );
                // idk do something
            }
            PokerEventType::DealCards {
                round,
                n_cards,
                cards,
            } => {
                if round != BettingRound::PREFLOP {
                    self.hand_state = self.hand_state.next_round();
                }
                println!("-------------------");
                println!("Dealing: {}", round);
                println!("Cards: {}", cards_to_str(&cards));
            }
            PokerEventType::RequestAction => {
                // apply valid action
                let action = self.get_action();
                // send action to server
                self.codec
                    .send(PokerEvent {
                        from: self.addr,
                        event: PokerEventType::SendAction { action },
                    })
                    .await?;
            }
            PokerEventType::AlertAction {
                action,
                player,
                pot,
                stacks,
                wagers,
            } => {
                self.hand_state = self.hand_state.apply_action(action);
                println!("-------------------");
                match player == self.position {
                    true => println!("Hero has chosen to {}", action),
                    false => println!("Villan has chosen to {}", action),
                }
                println!("Pot: {}", pot);
                println!(
                    "Hero stack: {} | Villan Stack: {}",
                    stacks[usize::from(self.position)],
                    stacks[usize::from(1 - self.position)]
                );
                println!(
                    "Hero wager: {} | Villan wager: {}",
                    wagers[usize::from(self.position)],
                    wagers[usize::from(1 - self.position)]
                );
            }
            _ => {}
        }
        Ok(())
    }
    fn get_action(&mut self) -> Action {
        // make sure that if chosen action is bet or raise,
        // the the bet or raise size makes sense and is valid
        let actions = self.hand_state.valid_actions();
        let stdin = std::io::stdin();
        let mut is_action_valid = false;

        println!("-------------------");
        println!("Please select an action.");

        while !is_action_valid {
            is_action_valid = true;
            // List Valid actions
            actions.iter().enumerate().for_each(|(i, a)| match a {
                Action::BET(_) => println!("{}: {}", i.to_string().red(), "Bet".bright_cyan()),
                Action::RAISE(_) => println!("{}: {}", i.to_string().red(), "Raise".bright_cyan()),
                Action::CALL => {
                    let call_amt = self.hand_state.other_player().wager()
                        - self.hand_state.current_player().wager();
                    println!(
                        "{}: {} {}",
                        i.to_string().red(),
                        "Call".bright_cyan(),
                        call_amt.to_string().bright_yellow()
                    );
                }
                Action::FOLD => println!("{}: {}", i.to_string().red(), "Fold".bright_cyan()),
                Action::CHECK => println!("{}: {}", i.to_string().red(), "Check".bright_cyan()),
            });
            // get input
            let mut input = String::new();
            stdin
                .lock()
                .read_line(&mut input)
                .expect("could not read line");
            // ensure input is a number in correct range
            let action_index = match input.trim().parse::<usize>() {
                Ok(num) => {
                    if num > actions.len() - 1 {
                        is_action_valid = false;
                        println!("Action must be between {} and {}", 0, actions.len() - 1);
                        continue;
                    }
                    num
                }
                Err(_) => {
                    is_action_valid = false;
                    println!("Input must be a number. Retrying.");
                    continue;
                }
            };
            // ensure bet is correct size
            if let Action::BET(_) = actions[action_index] {
                let max_bet = self.hand_state.current_player().stack();
                let min_bet = std::cmp::min(10, max_bet);
                let mut bet_s = String::new();
                println!("Input a bet size from ({}, {}): ", 1, max_bet);
                stdin.lock().read_line(&mut bet_s).unwrap();
                match bet_s.trim().parse::<u32>() {
                    Ok(num) => {
                        if num < min_bet || num > max_bet {
                            is_action_valid = false;
                            println!("Bet size out of range. Retrying");
                            continue;
                        }
                        if is_action_valid {
                            return Action::BET(num);
                        }
                    }
                    Err(_) => {
                        is_action_valid = false;
                        println!("Bet size invalid. Retrying");
                        continue;
                    }
                };
            }
            if let Action::RAISE(_) = actions[action_index] {
                let max_raise = self.hand_state.current_player().stack();
                let min_raise = 2 * self.hand_state.other_player().wager();
                let mut raise_s = String::new();
                // Go all in
                if self.hand_state.current_player().stack() < min_raise {
                    return Action::RAISE(self.hand_state.current_player().stack());
                }
                println!("Input a raise size from [{},{}]: ", min_raise, max_raise);
                stdin.lock().read_line(&mut raise_s).unwrap();
                match raise_s.trim().parse::<u32>() {
                    Ok(num) => {
                        if num < min_raise || num > max_raise {
                            is_action_valid = false;
                            println!("Raise size out of range. Retrying");
                            continue;
                        }
                        if is_action_valid {
                            return Action::RAISE(num);
                        }
                    }
                    Err(_) => {
                        is_action_valid = false;
                        println!("Raise size invalid. Retrying");
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

impl Stream for HumanAgent {
    type Item = Result<PokerEvent, PokerCodecError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Secondly poll the `Framed` stream.
        let result: Option<_> = futures::ready!(Pin::new(&mut self.codec).poll_next(cx));

        Poll::Ready(result)
    }
}
