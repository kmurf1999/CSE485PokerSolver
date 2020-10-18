/// Tree object in rust
/// 
/// Nodes are stored in a central arena
/// and are used to iterate using node indices instead of pointers
pub struct Tree<T> {
    nodes: Vec<Node<T>>
}

/// Node object for the tree
/// 
/// Children is a list of node indices instead of pointers
pub struct Node<T> {
    pub data: T,
    pub parent: Option<usize>,
    pub children: Vec<usize>
}

impl<T> Node<T> {
    pub fn add_child(&mut self, node_idx: usize) {
        self.children.push(node_idx);
    }
}

impl<T> Tree<T> {
    /// Create an empty tree
    pub fn new() -> Tree<T> {
        Tree {
            nodes: Vec::new()
        }
    }
    /// Add a new node into the tree
    pub fn add_node(&mut self, parent: Option<usize>, data: T) -> usize {
        let next_index = self.nodes.len();
        self.nodes.push(Node {
            parent,
            children: Vec::new(),
            data
        });
        return next_index;
    }
    pub fn get_node_mut(&mut self, node_idx: usize) -> &mut Node<T> {
        &mut self.nodes[node_idx]
    }
    pub fn get_node(&self, node_idx: usize) -> &Node<T> {
        &self.nodes[node_idx]
    }
}