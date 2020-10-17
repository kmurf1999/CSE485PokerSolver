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