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