use crate::node::NodeId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MultiExpr {
    pub child_ids: Vec<NodeId>,
}

impl MultiExpr {
    pub fn new(child_ids: impl Into<Vec<NodeId>>) -> Self {
        let child_ids = child_ids.into();
        Self { child_ids }
    }
}
