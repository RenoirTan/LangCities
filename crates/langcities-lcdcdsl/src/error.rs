use std::fmt::Display;

use langcities_common::error::LcError;

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

langcities_common::lc_error_trait! {
    DslErrorTrait for DslError as DslErrorKind {
        NodeNotFound, UnsupportedNode, BadValue,
    }
    declare {
        fn node_not_found_of(node_id: impl Into<NodeId>) -> Self;
        fn bad_value_of(value: impl Display) -> Self;
    }
    define {
        fn node_not_found_of(node_id: impl Into<NodeId>) -> Self {
            Self::node_not_found_desc(format!("Node {} not found", node_id.into()))
        }
        fn bad_value_of(value: impl Display) -> Self {
            Self::bad_value_desc(format!("Invalid value: {}", value))
        }
    }
}
