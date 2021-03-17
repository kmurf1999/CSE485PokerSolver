/// Tree object in rust
///
/// Nodes are stored in a central arena
/// and are used to iterate using node indices instead of pointers
pub struct Tree<T> {
    nodes: Vec<Node<T>>,
}

pub type NodeIndex = usize;

/// Node object for the tree
///
/// Children is a list of node indices instead of pointers
pub struct Node<T> {
    pub data: T,
    pub parent: Option<NodeIndex>,
    pub children: Vec<NodeIndex>,
}

impl<T> Node<T> {
    /// adds a child node index into the children array of this node
    pub fn add_child(&mut self, node_idx: NodeIndex) {
        self.children.push(node_idx);
    }
}

impl<T> Default for Tree<T> {
    fn default() -> Self {
        Tree { nodes: Vec::new() }
    }
}

impl<T> Tree<T> {
    /// Add a new node into the tree
    pub fn add_node(&mut self, parent: Option<NodeIndex>, data: T) -> NodeIndex {
        let next_index = self.nodes.len();
        self.nodes.push(Node {
            parent,
            children: Vec::new(),
            data,
        });
        next_index
    }
    /// get a mutable reference to a node at index
    pub fn get_node_mut(&mut self, node_idx: NodeIndex) -> &mut Node<T> {
        assert!(node_idx < self.nodes.len());
        &mut self.nodes[node_idx]
    }
    /// gets a reference to to a node at index
    pub fn get_node(&self, node_idx: NodeIndex) -> &Node<T> {
        assert!(node_idx < self.nodes.len());
        &self.nodes[node_idx]
    }
    /// Returns a preorder tree iterator
    pub fn iter(&self) -> TreeIter<T> {
        TreeIter::new(0, &self)
    }
}

/// Preorder tree iterator
pub struct TreeIter<'a, T> {
    stack: Vec<NodeIndex>,
    tree: &'a Tree<T>,
}

impl<'a, T> TreeIter<'a, T> {
    pub fn new(root: NodeIndex, tree: &'a Tree<T>) -> Self {
        TreeIter::<'a, T> {
            stack: vec![root],
            tree,
        }
    }
}

impl<'a, T> Iterator for TreeIter<'a, T> {
    type Item = &'a Node<T>;
    fn next(&mut self) -> Option<&'a Node<T>> {
        if let Some(node_idx) = self.stack.pop() {
            self.tree.get_node(node_idx).children.iter().for_each(|&n| {
                self.stack.push(n);
            });
            return Some(self.tree.get_node(node_idx));
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
