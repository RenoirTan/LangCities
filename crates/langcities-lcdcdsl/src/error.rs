use std::{error::Error, fmt::Display};

use crate::node::NodeId;

#[derive(Debug)]
pub struct DslError {
    pub source: Option<Box<dyn Error>>,
    pub kind: DslErrorKind,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DslErrorKind {
    NodeNotFound,
}

impl DslError {
    pub fn new(source: Option<Box<dyn Error>>, kind: impl Into<DslErrorKind>) -> Self {
        let kind = kind.into();
        Self { source, kind }
    }

    pub fn node_not_found(node_id: impl Into<NodeId>) -> Self {
        Self::new(
            Some(format!("Node {} not found", node_id.into()).into()),
            DslErrorKind::NodeNotFound,
        )
    }
}

impl Display for DslError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DslError({})", self.kind)
    }
}

impl Display for DslErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for DslError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_ref().map(|e| e.as_ref())
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}
