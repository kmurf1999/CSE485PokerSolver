mod game_handler;
pub mod handler;
pub mod ws;

use std::collections::{HashMap};
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::{ws::Message, Filter, Rejection};
use poker_solver::codec;

// using future mpsc because it implements StreamExt
// tokio's mpsc doesn't implement it anymore
use futures::channel::mpsc;

pub const STACK_SIZE: u32 = 10000;
pub const MAX_PLAYERS: usize = 2;
pub const MIN_PLAYERS: usize = 2;
pub const BLINDS: [u32; 2] = [10, 5];
pub const ACTION_TIMEOUT: u64 = 30;


#[derive(Debug, Clone)]
/// maintains client information
pub struct Client {
    /// channel to send client messages
    pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>,
    /// ID of game client is currently joined to
    pub game_id: String,
}

#[derive(Debug, Clone)]
/// maintains game information
pub struct Game {
    /// set of client ids that are playing in the game
    /// client_id, stack_size
    pub clients: Vec<(String, u32)>,
    /// used by clients to send messages to game loop
    pub sender: mpsc::UnboundedSender<codec::PokerMessage>,
}

// game_id -> Game
pub type Result<T> = std::result::Result<T, Rejection>;
pub type Games = Arc<RwLock<HashMap<String, Game>>>;
pub type Clients = Arc<RwLock<HashMap<String, Client>>>;

/// middleware to include games
pub fn with_games(games: Games) -> impl Filter<Extract = (Games,), Error = Infallible> + Clone {
    warp::any().map(move || games.clone())
}

/// middleware to include clients
pub fn with_clients(
    clients: Clients,
) -> impl Filter<Extract = (Clients,), Error = Infallible> + Clone {
    warp::any().map(move || clients.clone())
}