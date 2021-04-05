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
    /// Raise is a "by value"
    /// meaning amount of chips past the call value
    /// It's fine to have these as the same enum variant because Bet-Bet cannot occur and neither can Raise-Raise
    BetRaise(u32),
    /// A chance card dealing
    Chance([u8; 4]),
}

/// Sinces check/fold & call actions cannot appear together
/// and they always appear first, we can easily index them
pub const CHECK_CALL_IDX: usize = 0;
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
