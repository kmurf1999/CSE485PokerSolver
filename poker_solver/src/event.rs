use crate::action::Action;
use crate::round::BettingRound;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PokerEventType {
    /// Used to alert players that game has started
    HandStart {
        /// what seat the player is in
        position: u8,
        blinds: [u32; 2],
        stacks: [u32; 2],
    },
    HandOver {
        winner: u8,
        stacks: [u32; 2],
        pot: u32,
    },
    /// Used to alert players what their cards are
    DealCards {
        round: BettingRound,
        n_cards: usize,
        cards: Vec<u8>,
    },
    /// Used by the server to request that a player make a decision
    RequestAction,
    /// Used to alert all players that a player has taken an action
    AlertAction {
        action: Action,
        player: u8,
        pot: u32,
        stacks: [u32; 2],
        wagers: [u32; 2],
    },
    /// Used to alert the server that a player intends to take an action
    SendAction { action: Action },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PokerEvent {
    pub from: SocketAddr,
    pub event: PokerEventType,
}
