use crate::{mpsc, Clients, Event, Games};

use futures::stream::StreamExt;
use futures::SinkExt;

use warp::filters::ws::Message;

pub async fn game_loop(
    mut receiver: mpsc::UnboundedReceiver<Event>,
    game_id: String,
    games: Games,
    clients: Clients,
) {
    println!("game started");

    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));

    // let game = games.read().await.get(&game_id).unwrap().clone();

    loop {
        tokio::select! {
            packet = receiver.next() => match packet {
                Some(event) => {
                    println!("id: {}, msg: {:?}", event.from, event.message);
                }
                None => {
                    break;
                }
            },
            _ = interval.tick() => {
                let client_ids = games.read().await.get(&game_id).unwrap().clients.clone();
                for client_id in client_ids.iter() {
                    let clients = clients.read().await;
                    match clients.get(client_id).unwrap().sender.clone() {
                        Some(mut sender) => {
                            let ev = Event {
                                from: "server".into(),
                                message: "alive".into(),
                            };
                            sender.send(Ok(Message::text(serde_json::to_string(&ev).expect("json serialization error")))).await.expect("error")
                        }
                        None => (),
                    }


                }
            }
        }
    }

    println!("game ended");
}