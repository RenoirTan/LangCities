use std::collections::HashMap;

use crate::node::{Node, NodeId};

use super::context::TreeContext;

#[derive(Clone, Debug)]
pub struct Tree {
    pub context: TreeContext,
    pub arena: HashMap<NodeId, Node>,
    pub root_node_id: Option<NodeId>,
}

impl Tree {
    pub fn new<C, A, R>(context: C, arena: A, root_node_id: R) -> Self
    where
        C: Into<TreeContext>,
        A: Into<HashMap<NodeId, Node>>,
        R: Into<Option<NodeId>>,
    {
        let (context, arena, root_node_id) = (context.into(), arena.into(), root_node_id.into());
        Self {
            context,
            arena,
            root_node_id,
        }
    }
}
