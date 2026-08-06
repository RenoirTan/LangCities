use std::collections::HashMap;

use crate::{
    node::{Node, NodeId},
    tree::{Tree, TreeContext},
};

#[derive(Clone, Debug)]
pub struct TreeBuilder {
    pub next_node_id: NodeId,
    pub tree: Tree,
}

impl TreeBuilder {
    pub fn new(source: impl Into<String>) -> Self {
        let context = TreeContext::new(source);
        let tree = Tree::new(context, HashMap::new(), None);
        Self {
            next_node_id: 0,
            tree,
        }
    }

    pub fn get_next_node_id(&mut self) -> NodeId {
        let id = self.next_node_id;
        self.next_node_id += 1;
        id
    }

    pub fn register_node(&mut self, node: Node) -> Option<Node> {
        self.tree.arena.insert(node.context.node_id, node)
    }
}
