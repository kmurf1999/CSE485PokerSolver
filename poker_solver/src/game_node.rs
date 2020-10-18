use std::fmt;

use crate::action::Action;

pub enum TerminalType {
    AllIn,
    Showdown,
    Fold
}

pub enum GameNode {
    PrivateChance,
    PublicChance,
    Action {
        index: usize,
        actions: Vec<Action>
    },
    Terminal {
        value: u32,
        ttype: TerminalType,
        last_to_act: u8
    }
}

/// For printing actions to terminal
impl fmt::Display for TerminalType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TerminalType::AllIn => write!(f, "Allin"),
            TerminalType::Showdown => write!(f, "Showdown"),
            TerminalType::Fold => write!(f, "Fold")
        }
    }
}
