use poker_solver::game_node::GameNode;
use poker_solver::round::BettingRound;
use poker_solver::tree::Tree;
use poker_solver::tree_builder::{TreeBuilder, TreeBuilderOptions};
use std::iter::repeat;

/// Recursivly prints a node on the game tree
fn print_node(tree: &Tree<GameNode>, node: usize, depth: usize) {
    let n = tree.get_node(node);
    let spaces = repeat("  ").take(depth).collect::<String>();
    match &n.data {
        GameNode::PrivateChance => {
            println!("{}Deal private cards", spaces);
            print_node(tree, n.children[0], depth + 1);
        }
        GameNode::PublicChance => {
            println!("{}Deal public cards", spaces);
            print_node(tree, n.children[0], depth + 1);
        }
        GameNode::Terminal {
            ttype,
            value,
            last_to_act: _,
        } => {
            println!("{}{} - value: {}", spaces, ttype, value);
        }
        GameNode::Action {
            actions,
            index,
            player,
        } => {
            for (i, action) in actions.iter().enumerate() {
                println!("{}{}", spaces, action);
                print_node(tree, n.children[i], depth + 1);
            }
        }
    }
}

fn main() {
    let options = TreeBuilderOptions {
        blinds: None,
        stacks: [1000, 1000],
        pot: Some(100),
        round: Some(BettingRound::RIVER),
        bet_sizes: [vec![1.0], vec![1.0], vec![1.0], vec![1.0]],
        raise_sizes: [vec![], vec![], vec![], vec![1.0]],
    };
    let tree = TreeBuilder::build(&options);
    print_node(&tree, 0, 0);
}
