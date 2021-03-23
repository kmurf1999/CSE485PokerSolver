use poker_solver::action::Action;
use poker_solver::codec::{PokerEvent, PokerMessage};
use poker_solver::state::GameState;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use serde_json::json;
use std::collections::HashMap;
use websocket::client::ClientBuilder;
use websocket::sync::Client;
use websocket::{Message, OwnedMessage};

const BASE_URI: &str = "http://127.0.0.1:3001";

async fn client_loop(client: Client<std::net::TcpStream>) {
    let mut rng = thread_rng();
    let (mut receiver, mut sender) = client.split().unwrap();
    let mut hand_state = GameState::default();
    for message in receiver.incoming_messages() {
        let message = match message {
            Ok(m) => m,
            Err(e) => {
                println!("Receive Loop: {:?}", e);
                return;
            }
        };
        let data = match message {
            OwnedMessage::Text(data) => data,
            _ => {
                return;
            }
        };
        let message: PokerMessage = serde_json::from_str(&data).unwrap();
        println!("{:?}", message);
        match message.event {
            PokerEvent::HandStart { stacks, position } => {
                hand_state = GameState::init_with_blinds(stacks, [10, 5], None);
            }
            PokerEvent::AlertAction {
                action,
                wagers: _,
                stacks: _,
                pot: _,
            } => {
                hand_state = hand_state.apply_action(action);
                if hand_state.bets_settled() && !hand_state.is_game_over() {
                    hand_state = hand_state.next_round();
                }
            }
            PokerEvent::DealCards { cards: _, round: _ } => {
                hand_state.deal_cards();
            }
            PokerEvent::RequestAction => {
                let actions = hand_state.valid_actions();
                let action = actions.choose(&mut rng).unwrap();
                let current_stack = hand_state.current_player().stack();
                match action {
                    Action::BET(amt) => {
                        let min_bet = std::cmp::min(current_stack, 10);
                        let bet_size = rng.gen_range(min_bet,current_stack+1);
                        let action = Action::BET(bet_size);
                        sender
                            .send_message(&Message::from(OwnedMessage::from(
                                json!(PokerEvent::SendAction { action: action }).to_string(),
                            )))
                            .unwrap();
                    }
                    Action::RAISE(amt) => {
                        let min_raise = std::cmp::min(current_stack, hand_state.other_player().wager()*2);
                        let raise_size = rng.gen_range(min_raise,current_stack+1);
                        let action = Action::RAISE(raise_size);
                        sender
                            .send_message(&Message::from(OwnedMessage::from(
                                json!(PokerEvent::SendAction { action: action }).to_string(),
                            )))
                            .unwrap();
                    }
                    _ => {
                        sender
                            .send_message(&Message::from(OwnedMessage::from(
                                json!(PokerEvent::SendAction { action: *action }).to_string(),
                            )))
                            .unwrap();
                    }
                }
            }
            _ => {}
        }
    }
    println!("loop");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let join_res = client
        .post(&format!("{}/join", BASE_URI))
        .send()
        .await?
        .json::<HashMap<String, String>>()
        .await?;

    let client = ClientBuilder::new(join_res.get("url").unwrap())
        .unwrap()
        .add_protocol("rust-websocket")
        .connect_insecure()
        .unwrap();

    client_loop(client).await;

    Ok(())
}
