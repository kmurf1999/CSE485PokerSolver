mod game_handler;
pub mod handler;
pub mod ws;

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::{ws::Message, Filter, Rejection};

// using future mpsc because it implements StreamExt
// tokio's mpsc doesn't implement it anymore
use futures::channel::mpsc;

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    /// client id message was from, or none if from server
    from: String,
    message: String,
}

#[derive(Debug, Clone)]
pub struct Client {
    /// channel to send client messages
    pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>,
    /// ID of game client is currently joined to
    pub game_id: String,
}

#[derive(Debug, Clone)]
pub struct Game {
    /// set of client ids that are playing in the game
    pub clients: HashSet<String>,
    /// used by clients to send messages to game loop
    pub sender: mpsc::UnboundedSender<Event>,
}

// game_id -> Game
pub type Result<T> = std::result::Result<T, Rejection>;
pub type Games = Arc<RwLock<HashMap<String, Game>>>;
pub type Clients = Arc<RwLock<HashMap<String, Client>>>;

pub fn with_games(games: Games) -> impl Filter<Extract = (Games,), Error = Infallible> + Clone {
    warp::any().map(move || games.clone())
}

pub fn with_clients(
    clients: Clients,
) -> impl Filter<Extract = (Clients,), Error = Infallible> + Clone {
    warp::any().map(move || clients.clone())
}