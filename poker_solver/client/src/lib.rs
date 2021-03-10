// use clap::Clap;
use std::collections::HashMap;
use websocket::client::ClientBuilder;
use websocket::{OwnedMessage, Message};
use websocket::sync::Client;
use poker_solver::state::GameState;
use poker_solver::codec::{PokerEvent, PokerMessage};
use poker_solver::action::Action;
use rand::{Rng, thread_rng};
use rand::seq::SliceRandom;
use serde_json::json;


const BASE_URI: &str = "http://localhost:3001";

// #[derive(Clap)]
// #[clap(version = "1.0", author = "Kyle <kmurf1999@gmail.com>")]
// struct Opts {
//     #[clap(short, default_value = "none")]
//     game_id: String
// }

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
        println!("{}", data);
        // let message: PokerMessage = serde_json::from_str(&data).unwrap();
        // match message.event {
        //     PokerEvent::HandStart { stacks } => {
        //         hand_state = GameState::init_with_blinds(stacks, [10, 5], None);
        //     },
        //     PokerEvent::AlertAction { action, stacks: _, wagers: _, pot: _ } => {
        //         hand_state = hand_state.apply_action(action);
        //         if hand_state.bets_settled() && !hand_state.is_game_over() {
        //             hand_state = hand_state.next_round();
        //         }
        //     },
        //     PokerEvent::DealCards { cards: _ , round: _ } => {
        //         hand_state.deal_cards();
        //     }
        //     PokerEvent::RequestAction => {
        //         let actions = hand_state.valid_actions();
        //         let action = actions.choose(&mut rng).unwrap();
        //         match action {
        //             Action::BET(amt) => {
        //                 sender.send_message(&Message::from(OwnedMessage::from(json!(PokerEvent::SendAction {
        //                     action: *action
        //                 }).to_string()))).unwrap();
        //             },
        //             Action::RAISE(amt) => {
        //                 sender.send_message(&Message::from(OwnedMessage::from(json!(PokerEvent::SendAction {
        //                     action: *action
        //                 }).to_string()))).unwrap();
        //             },
        //             _ => {
        //                 sender.send_message(&Message::from(OwnedMessage::from(json!(PokerEvent::SendAction {
        //                     action: *action
        //                 }).to_string()))).unwrap();
        //             }
        //         }
        //     },
        //     _ => {}
        // }
    }
}

pub async fn client(game_id: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let game_id: String = match game_id {
        Some(game_id) => {
            game_id.to_string()
        },
        None => {
            let create_res = client.post(&format!("{}/create", BASE_URI))
            .send()
            .await?
            .json::<HashMap<String, String>>()
            .await?;    
            let game_id = create_res.get("game_id").unwrap();
            game_id.to_string()
        }
    };
    println!("game_id: {}", game_id);

    let mut join_body = HashMap::new();
    join_body.insert("game_id", game_id);

    let join_res = client.post(&format!("{}/join", BASE_URI))
        .json(&join_body)
        .send()
        .await?
        .json::<HashMap<String, String>>()
        .await?;
        
    let client = ClientBuilder::new(join_res.get("url").unwrap())
		.unwrap()
		.add_protocol("rust-websocket")
		.connect_insecure()
		.unwrap();


    tokio::spawn(async move {
        client_loop(client).await;
    });


    Ok(())
}
