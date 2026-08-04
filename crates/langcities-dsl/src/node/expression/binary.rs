use std::fmt::Display;

use crate::node::NodeId;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BinaryExpression {
    pub op: BinaryOp,
    pub left_id: NodeId,
    pub right_id: NodeId,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BinaryOp {
    Add,
}

impl BinaryExpression {
    pub fn new<O, L, R>(op: O, left_id: L, right_id: R) -> Self
    where
        O: Into<BinaryOp>,
        L: Into<NodeId>,
        R: Into<NodeId>,
    {
        let (op, left_id, right_id) = (op.into(), left_id.into(), right_id.into());
        Self {
            op,
            left_id,
            right_id,
        }
    }
}

impl Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                BinaryOp::Add => "+",
            },
        )
    }
}
