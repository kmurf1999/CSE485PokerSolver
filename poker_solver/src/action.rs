//!
//! Represents an action in a game of HULN Holdem'.
//! Actions are used to perform state transitions.
//! They include all player action and chance actions.
//!
use serde::{Deserialize, Serialize};

/// Represents a player action
#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum Action {
    /// Check/Fold action
    Fold,
    /// Call a Bet or raise
    CheckCall,
    /// Bet action
    /// value is in chips
    /// Raise is a "to value"
    /// meaning amount of chips past the call value
    BetRaise(u32),
    /// A chance card dealing
    Chance([u8; 4]),
}

/// Used to index the result of `state.valid_actions()`
pub const CHECK_CALL_IDX: usize = 0;
/// Used to index the result of `state.valid_actions()`
pub const FOLD_IDX: usize = 1;

/// List of available actions
///
/// Note: Bet and Raise sizes could be invalid
pub static ACTIONS: &[Action; 4] = &[
    Action::CheckCall,
    Action::Fold,
    Action::BetRaise(1),
    Action::Chance([52; 4]),
];
