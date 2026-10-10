use crate::{
    dependency::{DepAlias, DependencyBuilder},
    error::DslError,
    node::NodeId,
    tree::Tree,
};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Dependency {
    pub identifier: String,
    pub data: DepData,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct DepData {
    pub node_id: NodeId,
}

impl DepData {
    pub fn new(node_id: impl Into<NodeId>) -> Self {
        let node_id = node_id.into();
        Self { node_id }
    }
}

impl Dependency {
    pub fn new<I, D>(identifier: I, data: D) -> Self
    where
        I: Into<String>,
        D: Into<DepData>,
    {
        let (identifier, data) = (identifier.into(), data.into());
        Self { identifier, data }
    }

    pub fn to_dep_alias(&self) -> Result<DepAlias, DslError> {
        self.identifier.parse::<DepAlias>()
    }

    pub fn find_all_in(tree: &Tree) -> Result<Vec<Dependency>, DslError> {
        let mut builder = DependencyBuilder::new(tree)?;
        builder.find()?;
        Ok(builder.to_list())
    }
}
