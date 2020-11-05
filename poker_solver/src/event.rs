use crate::round::BettingRound;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PokerEventType {
    StartHand {
        stacks: [u32; 2],
    },
    DealCards {
        round: BettingRound,
        n_cards: usize,
        cards: Vec<u8>,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PokerEvent {
    pub from: SocketAddr,
    pub event: PokerEventType,
}
