use std::fmt;

/// The Current Betting Round a Texas Holdem game is in
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BettingRound {
    PREFLOP,
    FLOP,
    TURN,
    RIVER
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

impl From<BettingRound> for usize {
    fn from(round: BettingRound) -> Self {
        return match round {
            BettingRound::PREFLOP => 0,
            BettingRound::FLOP => 1,
            BettingRound::TURN => 2,
            BettingRound::RIVER => 3
        }
    }
}