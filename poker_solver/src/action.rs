use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents a player action
#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum Action {
    /// Bet size in chips
    BET(u32),
    /// Raise is a "by value"
    /// meaning amount of chips past the call value
    RAISE(u32),
    /// Fold action
    FOLD,
    /// Call a bet or raise
    CALL,
    /// Check
    CHECK,
}

/// For printing actions to terminal
impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return match self {
            Action::BET(size) => write!(f, "Bet {}", size),
            Action::RAISE(size) => write!(f, "Raise {}", size),
            Action::FOLD => write!(f, "Fold"),
            Action::CALL => write!(f, "Call"),
            Action::CHECK => write!(f, "Check"),
        };
    }
}

/// List of available actions
///
/// Note: Bet and Raise sizes are invalid
pub static ACTIONS: &'static [Action; 5] = &[
    Action::BET(1),
    Action::RAISE(1),
    Action::FOLD,
    Action::CALL,
    Action::CHECK,
];
