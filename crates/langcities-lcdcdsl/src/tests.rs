use crate::{
    node::{
        BinaryExpr, BinaryOp, FunctionCallExpr, IdentifierPrim, Node, NodeContext, NodeId,
        NodeKind, StringLiteralExpr, StringLiteralKind,
    },
    tree::{Tree, TreeBuilder},
};

pub(crate) fn create_tree() -> Tree {
    let mut builder = TreeBuilder::new("$f($g(a, b) + c)");
    let mut node_id = builder.get_next_node_id();
    builder.register_node(Node::new(
        NodeKind::IdentifierPrim(IdentifierPrim),
        NodeContext::new(node_id, 0..2),
    ));
    node_id = builder.get_next_node_id();
    builder.register_node(Node::new(
        NodeKind::IdentifierPrim(IdentifierPrim),
        NodeContext::new(node_id, 3..5),
    ));
    node_id = builder.get_next_node_id();
    builder.register_node(Node::new(
        NodeKind::StringLiteralExpr(StringLiteralExpr::new(StringLiteralKind::Unquoted)),
        NodeContext::new(node_id, 6..7),
    ));
    node_id = builder.get_next_node_id();
    builder.register_node(Node::new(
        NodeKind::StringLiteralExpr(StringLiteralExpr::new(StringLiteralKind::Unquoted)),
        NodeContext::new(node_id, 9..10),
    ));
    node_id = builder.get_next_node_id();
    builder.register_node(Node::new(
        NodeKind::FunctionCallExpr(FunctionCallExpr::new(1 as NodeId, [2, 3])),
        NodeContext::new(node_id, 3..11),
    ));
    node_id = builder.get_next_node_id();
    builder.register_node(Node::new(
        NodeKind::StringLiteralExpr(StringLiteralExpr::new(StringLiteralKind::Unquoted)),
        NodeContext::new(node_id, 14..15),
    ));
    node_id = builder.get_next_node_id();
    builder.register_node(Node::new(
        NodeKind::BinaryExpr(BinaryExpr::new(BinaryOp::Add, 4 as NodeId, 5 as NodeId)),
        NodeContext::new(node_id, 3..15),
    ));
    node_id = builder.get_next_node_id();
    builder.register_node(Node::new(
        NodeKind::FunctionCallExpr(FunctionCallExpr::new(0 as NodeId, [6])),
        NodeContext::new(node_id, 0..16),
    ));
    builder.tree.root_node_id = Some(7);
    builder.tree
}
