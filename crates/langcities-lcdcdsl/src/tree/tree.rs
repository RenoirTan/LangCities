use std::collections::HashMap;

use crate::node::{Node, NodeId};

use super::context::TreeContext;

#[derive(Clone, Debug)]
pub struct Tree {
    pub context: TreeContext,
    pub arena: HashMap<NodeId, Node>,
}

impl Tree {
    pub fn new(context: impl Into<TreeContext>, arena: impl Into<HashMap<NodeId, Node>>) -> Self {
        let (context, arena) = (context.into(), arena.into());
        Self { context, arena }
    }
}
