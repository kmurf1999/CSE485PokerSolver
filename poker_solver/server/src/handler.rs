use crate::{game_handler, mpsc, ws, Client, Clients, Game, Games, Result};
use tracing::{info, error, debug, instrument};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;
use tokio::time::{sleep, Duration};

use warp::{reject::Reject, reply::json, Reply};

const GAME_SIZE: usize = 2;

#[derive(Serialize, Debug)]
struct CreateGameResponse {
    game_id: String,
}

#[derive(Serialize, Debug)]
struct JoinGameResponse {
    client_id: String,
    url: String,
}

#[derive(Deserialize, Debug)]
pub struct JoinGameRequest {
    game_id: String,
}

#[derive(Debug, Error, Serialize, Deserialize)]
enum GameError {
    #[error("game full")]
    GameFull,
    #[error("game does not exist")]
    DoesNotExist,
}

impl Reject for GameError {}

/// Creates a new game object and spawns a new thread to handle the game
///
/// # Arguments
///
/// * `game_id` id of game to create for lookup
/// * `games`
async fn create_game(game_id: &str, games: &Games, clients: &Clients) {
    let (sender, receiver) = mpsc::unbounded();

    // store sender in the hashmap to use it by clients
    games.write().await.insert(
        game_id.into(),
        Game {
            clients: Vec::new(),
            sender,
        },
    );

    let gi: String = game_id.to_string();
    let game_id = game_id.into();
    let games = games.clone();
    let clients = clients.clone();

    tokio::spawn(async move {
        // use receiver here
        game_handler::game_handler(receiver, game_id, games, clients).await
    });
    tokio::spawn(async move {
        // use receiver here
        // sleep(Duration::from_secs(5)).await;
        client::client(Some(gi)).await.unwrap();
    });
}

/// Adds a given client_id to a specified game
async fn join_game(game_id: &str, client_id: &str, games: &Games) -> Result<()> {
    // TODO add already joined
    match games.write().await.get_mut(game_id) {
        Some(game) => {
            if game.clients.len() >= GAME_SIZE {
                Err(GameError::GameFull.into())
            } else {
                game.clients.push((client_id.into(), crate::STACK_SIZE));
                Ok(())
            }
        }
        None => Err(GameError::DoesNotExist.into()),
    }
}

/// Creates a new client and adds them to game
pub async fn create_client(client_id: &str, game_id: &str, clients: &Clients) {
    clients.write().await.insert(
        client_id.into(),
        Client {
            sender: None,
            game_id: game_id.into(),
        },
    );
}

/// Creates a new game
pub async fn create_game_handler(games: Games, clients: Clients) -> Result<impl Reply> {
    let game_id = Uuid::new_v4().to_string();

    create_game(&game_id, &games, &clients).await;

    info!("game: {} created", &game_id);
    Ok(json(&CreateGameResponse { game_id }))
}

#[instrument]
/// Route to join a game
pub async fn join_game_handler(
    body: JoinGameRequest,
    games: Games,
    clients: Clients,
) -> Result<impl Reply> {
    let game_id = body.game_id;
    let client_id = Uuid::new_v4().to_string();

    join_game(&game_id, &client_id, &games).await?;
    create_client(&client_id, &game_id, &clients).await;

    info!("game: {} joined by: {}", game_id, client_id);
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