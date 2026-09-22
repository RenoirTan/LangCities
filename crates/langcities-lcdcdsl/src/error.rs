use std::fmt::Display;

use langcities_common::error::{Error, LcError};

use crate::node::NodeId;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DslErrorKind {
    NodeNotFound,
    UnsupportedNode,
    BadValue,
}

impl Display for DslErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub type DslError = LcError<DslErrorKind>;

pub trait DslErrorTrait {
    fn node_not_found_of(node_id: impl Into<NodeId>) -> Self;
    fn node_not_found(source: impl Into<Error>) -> Self;
    fn unsupported_node(source: impl Into<Error>) -> Self;
    fn bad_value_of(value: impl Display) -> Self;
    fn bad_value(source: impl Into<Error>) -> Self;
}

impl DslErrorTrait for DslError {
    fn node_not_found_of(node_id: impl Into<NodeId>) -> Self {
        Self::node_not_found(format!("Node {} not found", node_id.into()))
    }

    fn node_not_found(source: impl Into<Error>) -> Self {
        Self::new(Some(source.into()), DslErrorKind::NodeNotFound)
    }

    fn unsupported_node(source: impl Into<Error>) -> Self {
        Self::new(Some(source.into()), DslErrorKind::UnsupportedNode)
    }

    fn bad_value_of(value: impl Display) -> Self {
        Self::bad_value(format!("Invalid value: {}", value))
    }

    fn bad_value(source: impl Into<Error>) -> Self {
        Self::new(Some(source.into()), DslErrorKind::BadValue)
    }
}
