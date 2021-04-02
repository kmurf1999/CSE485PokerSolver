use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt;
use thiserror::Error;

/// The Current Betting Round a Texas Holdem game is in
#[derive(Debug, Copy, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub enum BettingRound {
    Preflop,
    Flop,
    Turn,
    River,
}

#[derive(Debug, Error)]
pub enum BettingRoundError {
    #[error("round out of bounds")]
    OutOfBounds,
}

impl fmt::Display for BettingRound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let round_str = match self {
            BettingRound::Preflop => "Preflop",
            BettingRound::Flop => "Flop",
            BettingRound::Turn => "Turn",
            BettingRound::River => "River",
        };
        write!(f, "{}", round_str)
    }
}

impl TryFrom<usize> for BettingRound {
    type Error = BettingRoundError;
    fn try_from(round: usize) -> Result<BettingRound, BettingRoundError> {
        let br = match round {
            0 => BettingRound::Preflop,
            1 => BettingRound::Flop,
            2 => BettingRound::Turn,
            3 => BettingRound::River,
            _ => return Err(BettingRoundError::OutOfBounds),
        };
        Ok(br)
    }
}

impl From<BettingRound> for usize {
    fn from(round: BettingRound) -> Self {
        match round {
            BettingRound::Preflop => 0,
            BettingRound::Flop => 1,
            BettingRound::Turn => 2,
            BettingRound::River => 3,
        }
    }
}
