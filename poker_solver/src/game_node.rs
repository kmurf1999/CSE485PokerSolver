use std::fmt;

use crate::action::Action;

/// The type of terminal node
pub enum TerminalType {
    /// Atleast one player has no chips left
    AllIn,
    /// A player has called on the final round of betting
    Showdown,
    /// A player has folded
    Fold,
}

/// Tree structure used to model a poker game tree
pub enum GameNode {
    /// represents a private card dealing
    PrivateChance,
    /// represents a public card dealing
    PublicChance,
    /// a player action
    Action {
        /// action node index
        index: u32,
        /// what player is action
        player: u8,
        /// an array of possible actions to be taken from this node
        actions: Vec<Action>,
    },
    /// represents a final node (allin, fold, showdown)
    Terminal {
        /// value of the pot
        value: u32,
        /// type of node (all, fold, showdown)
        ttype: TerminalType,
        /// which player acted last (used for determining who folded)
        last_to_act: u8,
    },
}

/// For printing nodes to terminal
impl fmt::Display for TerminalType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TerminalType::AllIn => write!(f, "Allin"),
            TerminalType::Showdown => write!(f, "Showdown"),
            TerminalType::Fold => write!(f, "Fold"),
        }
    }
}
