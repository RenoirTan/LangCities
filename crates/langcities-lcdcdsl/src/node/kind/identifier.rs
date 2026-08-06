#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IdentifierPrim;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IdentifierExpr {
    pub prim: IdentifierPrim,
}
