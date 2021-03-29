use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt;
use thiserror::Error;

/// The Current Betting Round a Texas Holdem game is in
#[derive(Debug, Copy, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub enum BettingRound {
    PREFLOP,
    FLOP,
    TURN,
    RIVER,
}

#[derive(Debug, Error)]
pub enum BettingRoundError {
    #[error("round out of bounds")]
    OutOfBounds,
}

impl fmt::Display for BettingRound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let round_str = match self {
            BettingRound::PREFLOP => "Preflop",
            BettingRound::FLOP => "Flop",
            BettingRound::TURN => "Turn",
            BettingRound::RIVER => "River",
        };
        write!(f, "{}", round_str)
    }
}

impl TryFrom<usize> for BettingRound {
    type Error = BettingRoundError;
    fn try_from(round: usize) -> Result<BettingRound, BettingRoundError> {
        let br = match round {
            0 => BettingRound::PREFLOP,
            1 => BettingRound::FLOP,
            2 => BettingRound::TURN,
            3 => BettingRound::RIVER,
            _ => return Err(BettingRoundError::OutOfBounds),
        };
        Ok(br)
    }
}

impl From<BettingRound> for usize {
    fn from(round: BettingRound) -> Self {
        match round {
            BettingRound::PREFLOP => 0,
            BettingRound::FLOP => 1,
            BettingRound::TURN => 2,
            BettingRound::RIVER => 3,
        }
    }
}
