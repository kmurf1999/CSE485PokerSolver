use crate::{game_handler, mpsc, ws, Client, Clients, Game, Games, Result};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;
use warp::{reject::Reject, reply::json, Reply};

const GAME_SIZE: usize = 5;

#[derive(Serialize, Debug)]
struct CreateGameResponse {
    game_id: String,
}

#[derive(Serialize, Debug)]
struct JoinGameResponse {
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
async fn create_game(game_id: &str, games: &Games, clients: &Clients) {
    let (sender, receiver) = mpsc::unbounded();

    // store sender in the hashmap to use it by clients
    games.write().await.insert(
        game_id.into(),
        Game {
            clients: Default::default(),
            sender,
        },
    );

    let game_id = game_id.into();
    let games = games.clone();
    let clients = clients.clone();

    tokio::spawn(async move {
        // use receiver here
        game_handler::game_loop(receiver, game_id, games, clients).await
    });
}

/// Adds a given client_id to a specified game
async fn join_game(game_id: &str, client_id: &str, games: &Games) -> Result<()> {
    match games.write().await.get_mut(game_id) {
        Some(game) => {
            if game.clients.len() >= GAME_SIZE {
                Err(GameError::GameFull.into())
            } else {
                game.clients.insert(client_id.into());
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

pub async fn create_game_handler(games: Games, clients: Clients) -> Result<impl Reply> {
    let game_id = Uuid::new_v4().to_string();

    create_game(&game_id, &games, &clients).await;

    Ok(json(&CreateGameResponse { game_id }))
}

pub async fn join_game_handler(
    body: JoinGameRequest,
    games: Games,
    clients: Clients,
) -> Result<impl Reply> {
    let game_id = body.game_id;
    let client_id = Uuid::new_v4().to_string();

    join_game(&game_id, &client_id, &games).await?;
    create_client(&client_id, &game_id, &clients).await;

    Ok(json(&JoinGameResponse {
        url: format!("ws://127.0.0.1:8080/ws/{}", client_id),
    }))
}

pub async fn ws_handler(
    ws: warp::ws::Ws,
    client_id: String,
    games: Games,
    clients: Clients,
) -> Result<impl Reply> {
    if clients.read().await.contains_key(&client_id) {
        Ok(ws.on_upgrade(move |socket| ws::client_connection(socket, client_id, games, clients)))
    } else {
        Err(warp::reject::not_found())
    }
}