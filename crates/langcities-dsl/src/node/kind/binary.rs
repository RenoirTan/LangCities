use crate::node::NodeId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BinaryExpr {
    pub op: BinaryOp,
    pub left_id: NodeId,
    pub right_id: NodeId,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
}

impl BinaryExpr {
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
