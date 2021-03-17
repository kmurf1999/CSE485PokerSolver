use crate::{game_handler, mpsc, ws, Client, Clients, Game, Games, Lobby, Result};
use serde::{Deserialize, Serialize};
use tokio::time::{self, Duration};
use tracing::{debug, info, instrument};
use uuid::Uuid;
use warp::{reply::json, Reply};

#[derive(Serialize, Debug)]
struct JoinGameResponse {
    client_id: String,
    url: String,
}

#[derive(Deserialize, Debug)]
pub struct JoinGameRequest {
    game_id: String,
}

/// Creates a new client and adds them to game
pub async fn create_client(client_id: &str, clients: &Clients) {
    clients.write().await.insert(
        client_id.into(),
        Client {
            sender: None,
            game_id: None,
        },
    );
}

async fn join_lobby(client_id: &str, lobby: &Lobby) {
    lobby.write().await.push(client_id.into());
}

pub async fn lobby_handler(lobby: Lobby, games: Games, clients: Clients) {
    let mut interval = time::interval(Duration::from_secs(1));
    // loop until lobby has filled
    loop {
        interval.tick().await;
        let count = lobby.read().await.len();
        // once lobby has filled create and start a new game
        if count >= crate::MIN_PLAYERS {
            // create new game
            let game_id = Uuid::new_v4().to_string();
            let (sender, receiver) = mpsc::unbounded();
            let mut game = Game {
                clients: Vec::new(),
                sender,
            };
            // remove players from lobby and add them to the game
            for _ in 0..crate::MIN_PLAYERS {
                let client_id = lobby.write().await.pop().unwrap();
                game.clients.push(client_id.clone());
                let mut clients = clients.write().await;
                let client = clients.get_mut(&client_id).expect("not present");
                client.game_id = Some(game_id.clone());
            }
            // add game
            games.write().await.insert(game_id.clone(), game);
            // start loop
            let games = games.clone();
            let clients = clients.clone();
            tokio::spawn(async move {
                // use receiver here
                game_handler::game_handler(receiver, game_id, games, clients).await
            });
        }
    }
}

#[instrument]
/// Route to join a game
pub async fn join_handler(lobby: Lobby, clients: Clients) -> Result<impl Reply> {
    // let game_id = body.game_id;
    let client_id = Uuid::new_v4().to_string();

    create_client(&client_id, &clients).await;
    join_lobby(&client_id, &lobby).await;
    info!("client joined. client id: {}", &client_id);

    Ok(json(&JoinGameResponse {
        client_id: client_id.clone(),
        url: format!("ws://127.0.0.1:3001/ws/{}", client_id),
    }))
}

#[instrument]
/// Handles connection and sends it to websocket handler
pub async fn ws_handler(
    ws: warp::ws::Ws,
    client_id: String,
    games: Games,
    clients: Clients,
) -> Result<impl Reply> {
    if clients.read().await.contains_key(&client_id) {
        Ok(ws.on_upgrade(move |socket| ws::client_connection(socket, client_id, games, clients)))
    } else {
        debug!("tried to join game with invalid client id: {}", client_id);
        Err(warp::reject::not_found())
    }
}
