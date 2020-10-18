use crate::tree::Tree;
use crate::state::GameState;
use crate::game_node::{GameNode, TerminalType};
use crate::round::BettingRound;
use crate::action::Action;

pub struct TreeBuilderOptions {
    /// value of the blinds
    /// if this option is set, `pot` is ignored
    blinds: Option<[u32; 2]>,
    /// The initial stack size of each player
    stacks: [u32; 2],
    /// The initial pot size
    pot: Option<u32>,
    /// Initial betting round
    /// if this option is not set, it defaults to `BettingRound::PREFLOP`
    round: Option<BettingRound>
}

/// A helper class to build a game tree
/// starts from initial parameters set by using the `TreeBuilderOptions` struct
pub struct TreeBuilder<'a> {
    options: &'a TreeBuilderOptions,
    tree: Tree<GameNode>,
    an_count: usize
}

impl<'a> TreeBuilder<'a>{
    /// Build a tree using initial options
    /// and return tree
    pub fn build(options: &'a TreeBuilderOptions) -> Tree<GameNode> {
        let mut builder = TreeBuilder {
            tree: Tree::<GameNode>::new(),
            options,
            an_count: 0
        };
        // create initial state
        let initial_state = match options.blinds {
            Some(blinds) => {
                GameState::init_with_blinds(options.stacks, blinds, options.round)
            },
            None => {
                let pot = options.pot.unwrap();
                GameState::new(pot, options.stacks, options.round)
            }
        };
        // The root of the tree is always a private chance node
        builder.build_private_chance(initial_state);
        // return
        builder.tree
    }
    /// Create private chance node and recursivily build tree
    fn build_private_chance(&mut self, state: GameState) {
        let root = self.tree.add_node(None, GameNode::PrivateChance);
        let child = self.build_action_nodes(root, state);
        self.tree.get_node_mut(root).add_child(child);
    }
    /// Build action nodes recursivily and return index of action node
    fn build_action_nodes(&mut self, parent: usize, state: GameState) -> usize {
        // TODO add actions and round index
        let node = self.tree.add_node(Some(parent), GameNode::Action {
            index: self.an_count,
            actions: Vec::new()
        });
        // increment number of action nodes
        self.an_count += 1;
        // build each action
        state.valid_actions().iter().for_each(|action| {
            self.build_action(node, state, *action);
        });
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
        if let GameNode::Action { index: _, actions } = &mut self.tree.get_node_mut(parent).data {
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
            node = self.tree.add_node(Some(parent), GameNode::Terminal {
                ttype: TerminalType::Fold,
                last_to_act: folded,
                value: state.pot()
            });
        } else if let Some(_) = state.player_all_in() {
            node = self.tree.add_node(Some(parent), GameNode::Terminal {
                ttype: TerminalType::AllIn,
                last_to_act: 0,
                value: state.pot()
            }); 
        } else {
            node = self.tree.add_node(Some(parent), GameNode::Terminal {
                ttype: TerminalType::Showdown,
                last_to_act: 0,
                value: state.pot()
            }); 
        }
        node
    }
}