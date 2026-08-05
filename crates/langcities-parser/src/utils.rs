use tree_sitter::{Node as TSNode, Tree as TSTree};

#[derive(Clone, Debug)]
pub(crate) struct RawNodePath {
    pub indices: Vec<u32>,
}

impl RawNodePath {
    pub fn root() -> Self {
        Self {
            indices: Vec::new(),
        }
    }

    pub fn with_child(&self, index: u32) -> Self {
        let mut indices = self.indices.clone();
        indices.push(index);
        Self { indices }
    }

    pub fn of_tree<'t>(&self, tree: &'t TSTree) -> Option<TSNode<'t>> {
        let mut node = tree.root_node();
        for index in &self.indices {
            node = node.named_child(*index)?;
        }
        Some(node)
    }
}
