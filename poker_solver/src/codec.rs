use crate::action::Action;
use serde::{Deserialize, Serialize};

pub const STACK_SIZE: u32 = 10000;
pub const MAX_PLAYERS: usize = 2;
pub const MIN_PLAYERS: usize = 2;
pub const BLINDS: [u32; 2] = [10, 5];
pub const ACTION_TIMEOUT: u64 = 30;

#[derive(Debug, Serialize, Deserialize)]
pub enum PokerEvent {
    GameStart,
    GameEnd,
    HandStart {
        stacks: [u32; MAX_PLAYERS],
        position: [String; MAX_PLAYERS],
    },
    PostBlinds {
        stacks: [u32; MAX_PLAYERS],
        wagers: [u32; MAX_PLAYERS],
        blinds: [u32; MIN_PLAYERS],
        pot: u32,
    },
    RequestAction,
    SendAction {
        action: Action,
    },
    AlertAction {
        action: Action,
        wagers: [u32; MAX_PLAYERS],
        stacks: [u32; MAX_PLAYERS],
        pot: u32,
    },
    HandEnd {
        pot: u32,
        stacks: [u32; MAX_PLAYERS],
    },
    DealCards {
        round: crate::round::BettingRound,
        cards: Vec<u8>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PokerMessage {
    /// client id message was from, or none if from server
    pub from: Option<String>,
    /// game message
    pub event: PokerEvent,
}
