use crate::{mpsc, Clients, Event, Games};

use futures::future::FutureExt;
use futures::stream::StreamExt;
use futures::SinkExt;

use warp::ws::WebSocket;

/// Handles a msg from a client
/// async fn client_msg(id: &str, msg: Message, clients: &Clients, games: &Games) {
///     // we can force unwrap since we know client must exist
///     let game_id = clients.read().await.get(id).unwrap().game_id.clone();
///     println!("game id: {}", game_id);
///
///     println!("received message from {}: {:?}", id, msg);
/// }

/// Handles connection to a single client
pub async fn client_connection(ws: WebSocket, client_id: String, games: Games, clients: Clients) {
    let (client_ws_sender, mut client_ws_recv) = ws.split();
    let (client_sender, client_recv) = mpsc::unbounded();

    // forward messages to client
    tokio::task::spawn(client_recv.forward(client_ws_sender).map(|result| {
        if let Err(e) = result {
            eprintln!("error sending websocket msg: {}", e);
        }
    }));

    let game_id: String = {
        let mut clients = clients.write().await;
        let client = clients.get_mut(&client_id).expect("not present");
        client.sender = Some(client_sender);
        client.game_id.clone()
    };

    let mut game_sender = games.read().await.get(&game_id).unwrap().sender.clone();

    println!("{} connected", &client_id);

    // Recv loop
    // gets ws messages from client and forwards them to the game thread
    while let Some(result) = client_ws_recv.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                println!("error receiving ws message for {}: {}", &client_id, e);
                break;
            }
        };
        // get text data from message
        // TODO encode message as json game event
        let msg = match msg.to_str() {
            Ok(msg) => msg,
            Err(_) => {
                println!("no text data in message for {}", &client_id);
                break;
            }
        };

        // forward message to game server
        let event = Event {
            from: client_id.clone(),
            message: msg.into(),
        };

        match game_sender.send(event).await {
            Ok(()) => {
                // message send successfully
            }
            Err(_) => {
                println!("error forwarding message to game");
                break;
            }
        }
    }

    // remove client from game
    games
        .write()
        .await
        .get_mut(&game_id)
        .unwrap()
        .clients
        .remove(&client_id);
    // remove client
    clients.write().await.remove(&client_id);
    println!("{} disconnected", client_id);
}