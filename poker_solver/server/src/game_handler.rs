use crate::{mpsc, Clients, Games, Result};
use futures::SinkExt;
use poker_solver::action::Action;
use poker_solver::round::BettingRound;
use poker_solver::state::GameState;
use poker_solver::codec::{PokerEvent, PokerMessage};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use serde_json::json;
use thiserror::Error;
use tokio::time::{timeout, Duration};
use tracing::{info, instrument};
use warp::filters::ws::Message;
use warp::reject::Reject;


#[derive(Debug, Error, Serialize, Deserialize)]
enum GameError {
    #[error("send error")]
    SendError,
    #[error("recv error")]
    RecvError,
}

#[derive(Debug)]
struct GameRunner {
    receiver: mpsc::UnboundedReceiver<PokerMessage>,
    game_id: String,
    games: Games,
    clients: Clients,
}

impl Reject for GameError {}

impl GameRunner {
    #[instrument]
    /// waits for lobby to fill
    async fn wait_to_fill(&mut self) {
        // wait for both players to join
        info!("game: {}, waiting for lobby to fill", &self.game_id);
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
        loop {
            interval.tick().await;
            let game = self.games.read().await.get(&self.game_id).unwrap().clone();
            if game.clients.len() == crate::MIN_PLAYERS {
                info!("game: {}, started all players joined", &self.game_id);
                break;
            }
        }
    }
    /// returns true if the game is over (atleast one player has no chips)
    async fn is_game_over(&self) -> bool {
        let game = self.games.read().await.get(&self.game_id).unwrap().clone();
        for client in &game.clients {
            if client.1 == 0 {
                return true;
            }
        }
        return false;
    }
    /// returns a copy of all players stacks
    async fn get_player_stacks(&self) -> [u32; crate::MAX_PLAYERS] {
        let game = self.games.read().await.get(&self.game_id).unwrap().clone();
        let mut stacks = [0u32; crate::MAX_PLAYERS];
        for (i, c) in game.clients.iter().enumerate() {
            stacks[i] = c.1;
        }
        stacks
    }
    async fn set_player_stacks(&self, stacks: &[u32; crate::MAX_PLAYERS]) {
        let mut game = self.games.write().await.get_mut(&self.game_id).unwrap().clone();
        for (i, c) in &mut game.clients.iter_mut().enumerate() {
            c.1 = stacks[i];
        }
    }
    /// main function to play game
    async fn play_game(&mut self) -> Result<()> {
        // send game start
        self.broadcast_msg(PokerEvent::GameStart).await?;
        // loop until one player runs out of chips
        while !self.is_game_over().await {
            self.play_hand().await?;
        }
        // send game end
        self.broadcast_msg(PokerEvent::GameEnd).await?;
        Ok(())
    }
    async fn deal_cards(&self, hand_state: &mut poker_solver::state::GameState) -> Result<()> {
        hand_state.deal_cards();
        match hand_state.round() {
            BettingRound::PREFLOP => {
                for i in 0..crate::MAX_PLAYERS {
                    let player = hand_state.player(i);
                    self.send_msg(
                        PokerEvent::DealCards {
                            round: hand_state.round(),
                            cards: player.cards().to_vec(),
                        },
                        i,
                    )
                    .await?;
                }
            }
            _ => {
                self.broadcast_msg(PokerEvent::DealCards {
                    round: hand_state.round(),
                    cards: hand_state.board().to_vec(),
                })
                .await?
            }
        }
        Ok(())
    }
    /// runs a single hand
    async fn play_hand(&mut self) -> Result<()> {
        self.broadcast_msg(PokerEvent::HandStart {
            stacks: self.get_player_stacks().await
        }).await?;
        // update dealer position
        self.games
            .write()
            .await
            .get_mut(&self.game_id)
            .unwrap()
            .clients
            .rotate_right(1);
        // create hand state
        let mut hand_state = poker_solver::state::GameState::init_with_blinds(
            self.get_player_stacks().await,
            crate::BLINDS,
            None,
        );
        // tell players hand has started
        self.broadcast_msg(PokerEvent::PostBlinds {
            blinds: crate::BLINDS,
            stacks: hand_state.stacks(),
            wagers: hand_state.wagers(),
            pot: hand_state.pot(),
        })
        .await?;
        // deal cards
        self.deal_cards(&mut hand_state).await?;
        // loop until hand is finished
        while !hand_state.is_game_over() {
            // get action from player
            let action = match timeout(
                Duration::from_secs(crate::ACTION_TIMEOUT),
                self.request_action(hand_state),
            )
            .await
            {
                Ok(action) => action?,
                Err(_) => hand_state.default_action(),
            };
            // apply action
            hand_state = hand_state.apply_action(action);
            // alert players of action
            self.broadcast_msg(PokerEvent::AlertAction { action, pot: hand_state.pot(), wagers: hand_state.wagers(), stacks: hand_state.stacks() })
                .await?;
            // check if round is over
            if hand_state.bets_settled() && !hand_state.is_game_over() {
                // continue to next round
                hand_state = hand_state.next_round();
                self.deal_cards(&mut hand_state).await?;
            }
        }
        // final pot value
        let pot = hand_state.pot();
        // copy stack values after hand
        let mut stacks = [hand_state.player(0).stack(), hand_state.player(1).stack()];

        if let Some(player_fold) = hand_state.player_folded() {
            // player folded
            stacks[usize::from(player_fold) - 1] += pot;
        } else {
            // do showdown
            while hand_state.round() != BettingRound::RIVER {
                hand_state = hand_state.next_round();
                self.deal_cards(&mut hand_state).await?;
            }

            let board = hand_state.board();
            let player_0_score = poker_solver::card::score_hand(board, hand_state.player(0).cards());
            let player_1_score = poker_solver::card::score_hand(board, hand_state.player(1).cards());
            match player_0_score.cmp(&player_1_score) {
                Ordering::Less => {
                    // player 1 wins
                    stacks[1] += pot;
                },
                Ordering::Greater => {
                    // player 0 wins
                    stacks[0] += pot;
                },
                Ordering::Equal => {
                    // tie
                    stacks[0] += pot / 2;
                    stacks[1] += pot / 2;
                }
            }  
        }
        self.set_player_stacks(&stacks).await;

        // deal cards
        self.broadcast_msg(PokerEvent::HandEnd {
            stacks: self.get_player_stacks().await,
            pot
        }).await?;
        Ok(())
    }
    /// send message to all connected clients
    async fn broadcast_msg(&self, event: PokerEvent) -> Result<()> {
        let game = self.games.read().await.get(&self.game_id).unwrap().clone();
        let json_msg = json!(PokerMessage {
            from: None,
            event
        });
        for client in game.clients.iter() {
            let client_id = &client.0;
            let mut sender = self
                .clients
                .read()
                .await
                .get(client_id)
                .unwrap()
                .sender
                .clone()
                .unwrap();
            match sender.send(Ok(Message::text(json_msg.to_string()))).await {
                Ok(_) => {}
                Err(e) => {
                    return Err(GameError::SendError.into());
                }
            }
        }
        Ok(())
    }
    /// send message to all connected clients
    async fn send_msg(&self, event: PokerEvent, player_idx: usize) -> Result<()> {
        let game = self.games.read().await.get(&self.game_id).unwrap().clone();
        let json_msg = json!(PokerMessage {
            from: None,
            event
        });
        let client_id = &game.clients[player_idx].0;
        let mut sender = self
            .clients
            .read()
            .await
            .get(client_id)
            .unwrap()
            .sender
            .clone()
            .unwrap();
        match sender.send(Ok(Message::text(json_msg.to_string()))).await {
            Ok(_) => {}
            Err(e) => {
                return Err(GameError::SendError.into());
            }
        }
        Ok(())
    }
    /// request an action from a player
    async fn request_action(&mut self, hand_state: GameState) -> Result<Action> {
        // ask player for action
        let player_idx = usize::from(hand_state.current_player_idx());
        let client_id = self.games.read().await.get(&self.game_id).unwrap().clients[player_idx]
            .0
            .to_owned();
        self.send_msg(PokerEvent::RequestAction, player_idx).await?;
        loop {
            tokio::select! {
                packet = self.receiver.next() => match packet {
                    Some(event) => {
                        // check if sender is correct
                        if event.from != Some(client_id.clone()) {
                            continue;
                        }
                        if let PokerEvent::SendAction { action } = event.event {
                            if hand_state.is_action_valid(action) {
                                return Ok(action);
                            }
                            // invalid action
                        }
                    },
                    None => {
                        return Err(GameError::RecvError.into());
                    }
                },
            }
        }
    }
}

#[instrument]
pub async fn game_handler(
    receiver: mpsc::UnboundedReceiver<PokerMessage>,
    game_id: String,
    games: Games,
    clients: Clients,
) {
    let mut runner = GameRunner {
        receiver,
        game_id,
        games,
        clients,
    };
    runner.wait_to_fill().await;
    runner.play_game().await.unwrap();
}
