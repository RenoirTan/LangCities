use crate::node::NodeId;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct FunctionCall {
    pub identifier: NodeId,
    pub args: Vec<NodeId>,
}

impl FunctionCall {
    pub fn new<I, A>(identifier: I, args: A) -> Self
    where
        I: Into<NodeId>,
        A: Into<Vec<NodeId>>,
    {
        let (identifier, args) = (identifier.into(), args.into());
        Self { identifier, args }
    }
}
