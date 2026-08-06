use std::ops::Range;

use crate::{
    node::{NodeId, NodeKind},
    tree::Tree,
};

#[derive(Clone, Debug)]
pub struct NodeContext {
    pub node_id: NodeId,
    pub span: Range<usize>,
}

impl NodeContext {
    pub fn new<I, S, C>(node_id: I, span: S) -> Self
    where
        I: Into<NodeId>,
        S: Into<Range<usize>>,
    {
        let (node_id, span) = (node_id.into(), span.into());
        Self { node_id, span }
    }

    pub fn raw<'t>(&self, tree: &'t Tree) -> &'t str {
        &tree.context.source[self.span.clone()]
    }

    pub fn raw_mut<'t>(&self, tree: &'t mut Tree) -> &'t mut str {
        &mut tree.context.source[self.span.clone()]
    }

    pub fn node<'t>(&self, tree: &'t Tree) -> Option<&'t Node> {
        tree.arena.get(&self.node_id)
    }

    pub fn node_mut<'t>(&self, tree: &'t mut Tree) -> Option<&'t mut Node> {
        tree.arena.get_mut(&self.node_id)
    }
}

#[derive(Clone, Debug)]
pub struct Node {
    pub node: NodeKind,
    pub context: NodeContext,
}

impl Node {
    pub fn new<N, C>(node: N, context: C) -> Self
    where
        N: Into<NodeKind>,
        C: Into<NodeContext>,
    {
        let (node, context) = (node.into(), context.into());
        Self { node, context }
    }
}
