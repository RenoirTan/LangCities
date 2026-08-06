#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Dependency {
    pub identifier: String,
    pub kind: DependencyKind,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum DependencyKind {
    Func,
    Var,
}

impl Dependency {
    pub fn new<I, K>(identifier: I, kind: K) -> Self
    where
        I: Into<String>,
        K: Into<DependencyKind>,
    {
        let (identifier, kind) = (identifier.into(), kind.into());
        Self { identifier, kind }
    }
}
