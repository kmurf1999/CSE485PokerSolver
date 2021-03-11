use crate::{mpsc, Clients, Games};
use futures::future::FutureExt;
use futures::stream::StreamExt;
use futures::SinkExt;
use poker_solver::codec;
use tokio::time::{self, Duration};
use tracing::{error, info, instrument};

use warp::ws::{self, WebSocket};

/// Handles connection to a single client
/// forwards incoming client messages to the game server
#[instrument]
pub async fn client_connection(ws: WebSocket, client_id: String, games: Games, clients: Clients) {
    let (client_ws_sender, mut client_ws_recv) = ws.split();
    let (client_sender, client_recv) = mpsc::unbounded();

    // forward messages to client
    tokio::task::spawn(client_recv.forward(client_ws_sender).map(|result| {
        if let Err(e) = result {
            error!("error sending websocket msg: {}", e);
        }
    }));

    // wait for client to join game
    let mut interval = time::interval(Duration::from_secs(1));
    loop {
        interval.tick().await;
        if clients
            .read()
            .await
            .get(&client_id)
            .expect("not present")
            .game_id
            .is_some()
        {
            break;
        }
    }

    // TODO don't force unwrap
    clients.write().await.get_mut(&client_id).unwrap().sender = Some(client_sender);
    let game_id = clients
        .read()
        .await
        .get(&client_id)
        .unwrap()
        .game_id
        .as_ref()
        .unwrap()
        .clone();

    let mut game_sender = games.read().await.get(&game_id).unwrap().sender.clone();

    info!("{} connected", &client_id);

    // Recv loop
    // gets ws messages from client and forwards them to the game thread
    while let Some(result) = client_ws_recv.next().await {
        let msg: ws::Message = match result {
            Ok(msg) => msg,
            Err(e) => {
                error!("error receiving ws message for {}: {}", &client_id, e);
                break;
            }
        };
        // get text data from message
        let msg: &str = match msg.to_str() {
            Ok(msg) => msg,
            Err(_) => {
                error!("invalid data in message for {}", &client_id);
                break;
            }
        };
        // parse message as poker event
        let msg: codec::PokerEvent = match serde_json::from_str(msg) {
            Ok(msg) => msg,
            Err(_) => {
                error!("could not parse message from {} as poker data", &client_id);
                break;
            }
        };
        // forward message to game server
        let event = codec::PokerMessage {
            from: Some(client_id.clone()),
            event: msg,
        };

        match game_sender.send(event).await {
            Ok(()) => {
                // message send successfully
            }
            Err(_) => {
                error!("error forwarding message to game");
                break;
            }
        }
    }

    // remove client from game
    let player_idx = games
        .read()
        .await
        .get(&game_id)
        .unwrap()
        .clients
        .iter()
        .position(|c| *c == client_id)
        .unwrap();
    games
        .write()
        .await
        .get_mut(&game_id)
        .unwrap()
        .clients
        .remove(player_idx);

    // remove client
    clients.write().await.remove(&client_id);
    info!("{} disconnected", client_id);
}
