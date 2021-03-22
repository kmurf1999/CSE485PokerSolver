use crate::action::Action;
use crate::constants::*;
use crate::game_node::{GameNode, TerminalType};
use crate::round::BettingRound;
use crate::state::GameState;
use crate::tree::Tree;
use std::result::Result;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TreeBuilderError {
    #[error("too many players")]
    TooManyPlayers,
    #[error("too few players")]
    TooFewPlayers,
    #[error("invalid stacks")]
    InvalidStacks,
    #[error("invalid bet sizes")]
    InvalidBetSizes,
    #[error("invalid raise sizes")]
    InvalidRaiseSizes,
}

/// Options that specify how to build the game tree
pub struct TreeBuilderOptions {
    /// value of the blinds
    /// [big blind, small blind]
    /// if this option is set, `pot` is ignored
    pub blinds: Option<Vec<u32>>,
    /// The initial stack size of each player
    pub stacks: Vec<u32>,
    /// The initial pot size
    pub pot: u32,
    /// Initial betting round
    pub round: BettingRound,
    /// bet sizes expressed as a fraction of the pot
    /// An array of bet sizes for each player for each round
    pub bet_sizes: Vec<Vec<Vec<f64>>>,
    /// raise sizes as expressed as a fraction of the pot
    /// An array of raise sizes for each player for each round
    pub raise_sizes: Vec<Vec<Vec<f64>>>,
}

/// A helper class to build a game tree
/// starts from initial parameters set by using the `TreeBuilderOptions` struct
pub struct TreeBuilder<'a> {
    /// tree builder options
    options: &'a TreeBuilderOptions,
    /// the game tree
    tree: Tree<GameNode>,
    /// number of action nodes in the game tree
    action_node_count: u32,
}

impl<'a> TreeBuilder<'a> {
    /// Build a tree using initial options
    /// and return tree
    pub fn build(options: &'a TreeBuilderOptions) -> Result<Tree<GameNode>, TreeBuilderError> {
        let player_count = options.stacks.len();
        let blinds = options.blinds.clone();
        let stacks = options.stacks.clone();
        let pot = options.pot;
        let round = options.round;
        // how many rounds of play in this tree
        let round_count = 4 - usize::from(round);

        if player_count < MIN_PLAYERS {
            return Err(TreeBuilderError::TooFewPlayers);
        }
        if player_count > MAX_PLAYERS {
            return Err(TreeBuilderError::TooManyPlayers);
        }
        for stack in &stacks {
            if *stack == 0 {
                return Err(TreeBuilderError::InvalidStacks);
            }
        }
        if options.bet_sizes.len() != player_count {
            return Err(TreeBuilderError::InvalidBetSizes);
        }
        for player_bet_sizes in &options.bet_sizes {
            if player_bet_sizes.len() != round_count {
                return Err(TreeBuilderError::InvalidBetSizes);
            }
            for bet_sizes in player_bet_sizes {
                for bet_size in bet_sizes {
                    if *bet_size <= 0.0 {
                        return Err(TreeBuilderError::InvalidBetSizes);
                    }
                }
            }
        }
        if options.raise_sizes.len() != player_count {
            return Err(TreeBuilderError::InvalidRaiseSizes);
        }
        for player_raise_sizes in &options.raise_sizes {
            if player_raise_sizes.len() != round_count {
                return Err(TreeBuilderError::InvalidRaiseSizes);
            }
            for raise_sizes in player_raise_sizes {
                for raise_size in raise_sizes {
                    if *raise_size <= 0.0 {
                        return Err(TreeBuilderError::InvalidRaiseSizes);
                    }
                }
            }
        }

        let mut builder = TreeBuilder {
            options,
            tree: Tree::<GameNode>::default(),
            action_node_count: 0,
        };

        let initial_state = match &blinds {
            Some(blinds) => GameState::init_with_blinds(stacks, blinds.to_vec(), Some(round)),
            None => GameState::new(pot, stacks, Some(round)),
        };
        // The root of the tree is always a private chance node
        builder.build_private_chance(initial_state);
        // return
        Ok(builder.tree)
    }
    // Create private chance node and recursivily build tree
    fn build_private_chance(&mut self, state: GameState) {
        let root = self.tree.add_node(None, GameNode::PrivateChance);
        let child = self.build_action_nodes(root, state);
        self.tree.get_node_mut(root).add_child(child);
    }
    /// Build action nodes recursivily and return index of action node
    fn build_action_nodes(&mut self, parent: usize, state: GameState) -> usize {
        // TODO add actions and round index
        let player = state.current_player_idx();
        let node = self.tree.add_node(
            Some(parent),
            GameNode::Action {
                index: self.action_node_count,
                player,
                actions: Vec::new(),
            },
        );
        // increment number of action nodes
        self.action_node_count += 1;
        // build each action
        let valid_actions = state.valid_actions();
        let round = usize::from(state.round()) - usize::from(self.options.round);
        let pot = state.pot() as f64;
        for action in valid_actions {
            if let Action::BET(_) = action {
                // apply each bet size
                for size in &self.options.bet_sizes[usize::from(player)][round] {
                    let amt = (size * pot) as u32;
                    let action_with_size = Action::BET(amt);
                    if state.is_action_valid(action_with_size) {
                        self.build_action(node, state, action_with_size);
                    }
                }
            } else if let Action::RAISE(_) = action {
                // apply each raise size
                for size in &self.options.raise_sizes[usize::from(player)][round] {
                    let amt = (size * pot) as u32;
                    let action_with_size = Action::RAISE(amt);
                    if state.is_action_valid(action_with_size) {
                        self.build_action(node, state, action_with_size);
                    }
                }
            } else {
                self.build_action(node, state, action);
            }
        }
        node
    }
    /// Build a single action node
    fn build_action(&mut self, parent: usize, state: GameState, action: Action) {
        let next_state = state.apply_action(action);
        let child;
        if next_state.bets_settled() {
            if next_state.is_game_over() {
                // build terminal node
                child = self.build_terminal(parent, next_state)
            } else {
                // deal public chance
                child = self.build_public_chance(parent, state.next_round());
            }
        } else {
            child = self.build_action_nodes(parent, next_state);
        }
        // link new node to tree
        self.tree.get_node_mut(parent).add_child(child);
        // add action
        if let GameNode::Action {
            index: _,
            actions,
            player: _,
        } = &mut self.tree.get_node_mut(parent).data
        {
            actions.push(action);
        }
    }
    /// Build a public chance node and return node index
    fn build_public_chance(&mut self, parent: usize, state: GameState) -> usize {
        let node = self.tree.add_node(Some(parent), GameNode::PublicChance);
        let child = self.build_action_nodes(node, state);
        self.tree.get_node_mut(node).add_child(child);
        node
    }
    /// Build a terminal node and return node index
    fn build_terminal(&mut self, parent: usize, state: GameState) -> usize {
        let node;
        if let Some(folded) = state.player_folded() {
            node = self.tree.add_node(
                Some(parent),
                GameNode::Terminal {
                    ttype: TerminalType::Fold,
                    last_to_act: folded,
                    value: state.pot(),
                },
            );
        } else if state.player_all_in().is_some() {
            node = self.tree.add_node(
                Some(parent),
                GameNode::Terminal {
                    ttype: TerminalType::AllIn,
                    last_to_act: 0,
                    value: state.pot(),
                },
            );
        } else {
            node = self.tree.add_node(
                Some(parent),
                GameNode::Terminal {
                    ttype: TerminalType::Showdown,
                    last_to_act: 0,
                    value: state.pot(),
                },
            );
        }
        node
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build() {
        let options = TreeBuilderOptions {
            blinds: None,
            stacks: vec![10000, 10000],
            pot: 100,
            round: BettingRound::FLOP,
            bet_sizes: vec![
                vec![vec![0.5], vec![0.5], vec![1.0]],
                vec![vec![0.5], vec![0.5], vec![1.0]],
            ],
            raise_sizes: vec![
                vec![vec![1.0], vec![1.0], vec![1.0]],
                vec![vec![1.0], vec![1.0], vec![1.0]],
            ],
        };
        let tree = TreeBuilder::build(&options).unwrap();
        assert_eq!(tree.len(), 466);
    }
}
