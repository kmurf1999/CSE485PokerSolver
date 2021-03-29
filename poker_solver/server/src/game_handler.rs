use crate::{mpsc, Clients, Games, Result};
use futures::SinkExt;
use futures::StreamExt;
use poker_solver::action::Action;
use poker_solver::codec::{PokerEvent, PokerMessage};
use poker_solver::round::BettingRound;
use poker_solver::state::GameState;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::cmp::Ordering;
use thiserror::Error;
use tokio::time::sleep;
use tokio::time::{timeout, Duration};
use tracing::{debug, info, instrument};
use warp::filters::ws::Message;
use warp::header::value;
use warp::reject::Reject;

#[derive(Debug, Error, Serialize, Deserialize)]
enum GameError {
    #[error("send error")]
    SendError,
    #[error("recv error")]
    RecvError,
    #[error("game not found")]
    NotFound,
}

#[derive(Debug)]
struct GameRunner {
    /// for receiving messages from clients
    receiver: mpsc::UnboundedReceiver<PokerMessage>,
    /// for sending messages to clients
    clients: Clients,
    /// (client_id, stack_size)
    stacks: [u32; crate::MAX_PLAYERS],
    client_ids: [String; crate::MAX_PLAYERS],
}

impl Reject for GameError {}

impl GameRunner {
    fn new(receiver: mpsc::UnboundedReceiver<PokerMessage>, clients: Clients) -> Self {
        GameRunner {
            receiver,
            clients,
            stacks: [0; crate::MAX_PLAYERS],
            client_ids: Default::default(),
        }
    }
    #[instrument]
    /// waits for lobby to fill and create initial stacks
    async fn wait_to_fill(&mut self, games: Games, game_id: String) -> Result<()> {
        // wait for both players to join
        info!("waiting for lobby to fill. game id: {}", &game_id);
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
        loop {
            interval.tick().await;
            match games.read().await.get(&game_id) {
                Some(game) => {
                    if game.clients.len() >= crate::MIN_PLAYERS {
                        info!("game started all players joined. game id: {}", &game_id);
                        break;
                    }
                }
                None => {
                    return Err(GameError::NotFound.into());
                }
            }
        }
        // create stacks
        match games.read().await.get(&game_id) {
            Some(game) => {
                game.clients.iter().enumerate().for_each(|(i, client_id)| {
                    self.stacks[i] = crate::STACK_SIZE;
                    self.client_ids[i] = client_id.into();
                });
            }
            None => {
                return Err(GameError::NotFound.into());
            }
        }
        // bad way to do this but we want to wait until ws.rs sets the sender on each client
        interval.tick().await;
        Ok(())
    }
    /// returns true if the game is over (atleast one player has no chips)
    fn is_game_over(&self) -> bool {
        // TODO what if the end of the buffer is 0
        self.stacks.iter().any(|s| *s == 0)
    }
    /// main function to play game
    async fn play_game(&mut self) -> Result<()> {
        // send game start
        self.broadcast_msg(PokerEvent::GameStart).await?;
        // loop until one player runs out of chips
        while !self.is_game_over() {
            self.play_hand().await?;
            self.stacks.rotate_right(1);
            self.client_ids.rotate_right(1);
        }
        // send game end
        self.broadcast_msg(PokerEvent::GameEnd).await?;
        Ok(())
    }
    async fn deal_cards(&self, hand_state: &mut poker_solver::state::GameState) -> Result<()> {
        hand_state.deal_cards();
        match hand_state.round() {
            BettingRound::PREFLOP => {
                // TODO handle current player count instead of only max players
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
            stacks: self.stacks,
            position: self.client_ids.clone(),
        })
        .await?;
        // create hand state
        let mut hand_state =
            poker_solver::state::GameState::init_with_blinds(self.stacks, crate::BLINDS, None);
        // tell players hand has started
        self.broadcast_msg(PokerEvent::PostBlinds {
            blinds: crate::BLINDS,
            stacks: hand_state.stacks(),
            wagers: hand_state.wagers(),
            pot: hand_state.pot(),
        })
        .await?;
        sleep(Duration::from_millis(500)).await;
        // deal cards
        self.deal_cards(&mut hand_state).await?;
        sleep(Duration::from_millis(500)).await;
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
            self.broadcast_msg(PokerEvent::AlertAction {
                action,
                pot: hand_state.pot(),
                wagers: hand_state.wagers(),
                stacks: hand_state.stacks(),
            })
            .await?;
            sleep(Duration::from_millis(500)).await;
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
        let mut stacks = [0u32; crate::MAX_PLAYERS];
        stacks[0] = hand_state.player(0).stack();
        stacks[1] = hand_state.player(1).stack();

        if let Some(player_fold) = hand_state.player_folded() {
            // player folded
            stacks[1 - usize::from(player_fold)] += pot;
        } else {
            // do showdown
            while hand_state.round() != BettingRound::RIVER {
                hand_state = hand_state.next_round();
                self.deal_cards(&mut hand_state).await?;
            }

            let board = hand_state.board();
            let player_0_score =
                poker_solver::card::score_hand(board, hand_state.player(0).cards());
            let player_1_score =
                poker_solver::card::score_hand(board, hand_state.player(1).cards());
            match player_0_score.cmp(&player_1_score) {
                Ordering::Less => {
                    // player 1 wins
                    stacks[1] += pot;
                }
                Ordering::Greater => {
                    // player 0 wins
                    stacks[0] += pot;
                }
                Ordering::Equal => {
                    // tie
                    stacks[0] += pot / 2;
                    stacks[1] += pot / 2;
                }
            }

        }

        self.stacks = stacks;

        // deal cards
        self.broadcast_msg(PokerEvent::HandEnd {
            stacks: self.stacks,
            pot,
            player0_cards: hand_state.player(0).cards().to_vec(),
            player1_cards: hand_state.player(1).cards().to_vec(),
        })
        .await?;
        // wait for 5 seconds after hand ends
        sleep(Duration::from_millis(5000)).await;
        Ok(())
    }
    /// send message to all connected clients
    #[instrument]
    async fn broadcast_msg(&self, event: PokerEvent) -> Result<()> {
        let json_msg = json!(PokerMessage { from: None, event });
        for client_id in &self.client_ids {
            let mut sender = match self.clients.read().await.get(client_id) {
                Some(client) => client.sender.clone().unwrap(),
                None => {
                    return Err(GameError::SendError.into());
                }
            };
            if let Err(e) = sender.send(Ok(Message::text(json_msg.to_string()))).await {
                debug!("error in broadcast msg: {:?}", e);
                return Err(GameError::SendError.into());
            }
        }
        Ok(())
    }
    /// send message to all connected clients
    #[instrument]
    async fn send_msg(&self, event: PokerEvent, player_idx: usize) -> Result<()> {
        let json_msg = json!(PokerMessage { from: None, event });
        let mut sender = match self.clients.read().await.get(&self.client_ids[player_idx]) {
            Some(client) => client.sender.clone().unwrap(),
            None => {
                return Err(GameError::SendError.into());
            }
        };
        if let Err(e) = sender.send(Ok(Message::text(json_msg.to_string()))).await {
            debug!("error in send_msg: {:?}", e);
            return Err(GameError::SendError.into());
        }
        Ok(())
    }
    /// request an action from a player
    async fn request_action(&mut self, hand_state: GameState) -> Result<Action> {
        // ask player for action
        let player_idx = usize::from(hand_state.current_player_idx());
        self.send_msg(PokerEvent::RequestAction, player_idx).await?;
        loop {
            tokio::select! {
                packet = self.receiver.next() => match packet {
                    Some(event) => {
                        // check if sender is correct
                        if event.from != Some(self.client_ids[player_idx].clone()) {
                            continue;
                        }
                        if let PokerEvent::SendAction {action} = event.event {
                        println!("{}", json!(PokerEvent::SendAction {action}));
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
    let mut runner = GameRunner::new(receiver, clients);
    runner
        .wait_to_fill(games.clone(), game_id.clone())
        .await
        .unwrap();
    if let Err(e) = runner.play_game().await {
        // remove game when games over
        debug!("error in game_handler: {:?}", e);
        games.write().await.remove(&game_id);
    }
    info!("game completed. game_id: {}", &game_id);
}
