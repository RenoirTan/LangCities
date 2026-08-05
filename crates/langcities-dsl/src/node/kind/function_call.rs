use crate::node::NodeId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionCallExpr {
    pub identifier_id: NodeId,
    pub arg_ids: Vec<NodeId>,
}

impl FunctionCallExpr {
    pub fn new<I, A>(identifier_id: I, arg_ids: A) -> Self
    where
        I: Into<NodeId>,
        A: Into<Vec<NodeId>>,
    {
        let (identifier_id, arg_ids) = (identifier_id.into(), arg_ids.into());
        Self {
            identifier_id,
            arg_ids,
        }
    }
}
