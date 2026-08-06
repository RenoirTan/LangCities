use crate::{
    node::{
        BinaryExpr, BinaryOp, FunctionCallExpr, IdentifierPrim, Node, NodeContext, NodeId,
        NodeKind, StringLiteralExpr, StringLiteralKind,
    },
    tree::{Tree, TreeBuilder},
};

pub(crate) fn create_tree_0() -> Tree {
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

pub(crate) fn create_tree_1() -> Tree {
    let mut builder = TreeBuilder::new(
        "$mt.sc.ot_mt($ot.sc.pd_ot(something + \"more\")) + $mt.sc.ot_mt($identifier)",
    );
    let mut node_id = builder.get_next_node_id();
    builder.register_node(Node::new(
        NodeKind::IdentifierPrim(IdentifierPrim),
        NodeContext::new(node_id, 0..12),
    ));
    node_id = builder.get_next_node_id();
    builder.register_node(Node::new(
        NodeKind::IdentifierPrim(IdentifierPrim),
        NodeContext::new(node_id, 13..25),
    ));
    node_id = builder.get_next_node_id();
    builder.register_node(Node::new(
        NodeKind::StringLiteralExpr(StringLiteralExpr::new(StringLiteralKind::Unquoted)),
        NodeContext::new(node_id, 26..35),
    ));
    node_id = builder.get_next_node_id();
    builder.register_node(Node::new(
        NodeKind::StringLiteralExpr(StringLiteralExpr::new(StringLiteralKind::Dquoted)),
        NodeContext::new(node_id, 38..44),
    ));
    node_id = builder.get_next_node_id();
    builder.register_node(Node::new(
        NodeKind::BinaryExpr(BinaryExpr::new(BinaryOp::Add, 2 as NodeId, 3 as NodeId)),
        NodeContext::new(node_id, 26..44),
    ));
    node_id = builder.get_next_node_id();
    builder.register_node(Node::new(
        NodeKind::FunctionCallExpr(FunctionCallExpr::new(1 as NodeId, [4])),
        NodeContext::new(node_id, 13..45),
    ));
    node_id = builder.get_next_node_id();
    builder.register_node(Node::new(
        NodeKind::IdentifierPrim(IdentifierPrim),
        NodeContext::new(node_id, 49..61),
    ));
    node_id = builder.get_next_node_id();
    builder.register_node(Node::new(
        NodeKind::IdentifierExpr(Default::default()),
        NodeContext::new(node_id, 62..73),
    ));
    node_id = builder.get_next_node_id();
    builder.register_node(Node::new(
        NodeKind::FunctionCallExpr(FunctionCallExpr::new(6 as NodeId, [7])),
        NodeContext::new(node_id, 49..74),
    ));
    node_id = builder.get_next_node_id();
    builder.register_node(Node::new(
        NodeKind::BinaryExpr(BinaryExpr::new(BinaryOp::Add, 5 as NodeId, 8 as NodeId)),
        NodeContext::new(node_id, 0..74),
    ));
    builder.tree.root_node_id = Some(9);
    builder.tree
}
