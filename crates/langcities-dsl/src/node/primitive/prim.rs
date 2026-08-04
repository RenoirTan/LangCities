use super::IdentifierPrimitive;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PrimitiveNode {
    pub prim: Prim,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Prim {
    Identifier(IdentifierPrimitive),
}
