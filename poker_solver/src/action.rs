use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents a player action
#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum Action {
    /// Check/Fold action
    CheckFold,
    /// Call a Bet or raise
    Call,
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
pub const CHECK_FOLD_IDX: usize = 0;
pub const CALL_IDX: usize = 1;

/// For printing actions to terminal
///
/// # Example
///
/// ```
/// use poker_solver::action::Action;
/// println!("{}", Action::Fold);
/// ```
impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return match self {
            Action::BetRaise(size) => write!(f, "B({})", size),
            Action::CheckFold => write!(f, "F"),
            Action::Call => write!(f, "C"),
            Action::Chance(_) => write!(f, "D"),
        };
    }
}

/// List of available actions
///
/// Note: Bet and Raise sizes could be invalid
pub static ACTIONS: &[Action; 4] = &[
    Action::CheckFold,
    Action::Call,
    Action::BetRaise(1),
    Action::Chance([52; 4]),
];
