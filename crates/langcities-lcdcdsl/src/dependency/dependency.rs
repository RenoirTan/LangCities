use crate::{dependency::DepAlias, error::DslError};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Dependency {
    pub identifier: String,
    pub kind: DependencyUseKind,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum DependencyUseKind {
    Func,
    Var,
}

impl Dependency {
    pub fn new<I, U>(identifier: I, use_kind: U) -> Self
    where
        I: Into<String>,
        U: Into<DependencyUseKind>,
    {
        let (identifier, kind) = (identifier.into(), use_kind.into());
        Self { identifier, kind }
    }

    pub fn to_dep_alias(&self) -> Result<DepAlias, DslError> {
        self.identifier.parse::<DepAlias>()
    }
}
