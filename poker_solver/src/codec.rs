use crate::action::Action;
use crate::constants::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
/// Structure used send and receive game events to and from the server
/// encodes to json
pub enum PokerEvent {
    /// Event sent to clients when game starts
    GameStart,
    /// Event sent to clients when game ends
    GameEnd,
    /// Event sent to clients when hand starts
    HandStart {
        stacks: [u32; MAX_PLAYERS],
        position: [String; MAX_PLAYERS],
    },
    /// Event send to clients when blinds are posted
    PostBlinds {
        stacks: [u32; MAX_PLAYERS],
        wagers: [u32; MAX_PLAYERS],
        blinds: [u32; MIN_PLAYERS],
        pot: u32,
    },
    /// Event sent to a single client when they need to make an action
    RequestAction,
    /// Event send to the server from the client when the client takes an action
    SendAction { action: Action },
    /// Event sent to all clients when a client action has been confirmed
    AlertAction {
        action: Action,
        wagers: [u32; MAX_PLAYERS],
        stacks: [u32; MAX_PLAYERS],
        pot: u32,
    },
    /// Event sent to clients when hand has ended
    HandEnd {
        pot: u32,
        stacks: [u32; MAX_PLAYERS],
    },
    /// Event send to client or clients when cards are dealt
    DealCards {
        round: crate::round::BettingRound,
        cards: Vec<u8>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
/// Message which includes the sender's `client_id`
pub struct PokerMessage {
    /// client id message was from, or none if from server
    pub from: Option<String>,
    /// game message
    pub event: PokerEvent,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::Action;
    use serde_json::json;

    #[test]
    fn test_send_action_to_string() {
        assert_eq!(
            json!(PokerEvent::SendAction {
                action: Action::BET(10),
            })
            .to_string(),
            "{\"SendAction\":{\"action\":{\"BET\":10}}}"
        );
        assert_eq!(
            json!(PokerEvent::SendAction {
                action: Action::FOLD,
            })
            .to_string(),
            "{\"SendAction\":{\"action\":\"FOLD\"}}"
        );
    }
}
