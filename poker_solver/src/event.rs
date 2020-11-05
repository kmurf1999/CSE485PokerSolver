use serde::{Serialize, Deserialize};
use std::net::SocketAddr;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum PokerEventType {
    StartGame
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct PokerEvent {
    pub from: SocketAddr,
    pub event: PokerEventType
}