use std::ops::{Deref, DerefMut};

use crate::node::IdentifierPrimitive;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct IdentifierExpression {
    pub prim: IdentifierPrimitive,
}

impl<P: Into<IdentifierPrimitive>> From<P> for IdentifierExpression {
    fn from(prim: P) -> Self {
        let prim = prim.into();
        Self { prim }
    }
}

impl Deref for IdentifierExpression {
    type Target = IdentifierPrimitive;

    fn deref(&self) -> &Self::Target {
        &self.prim
    }
}

impl DerefMut for IdentifierExpression {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.prim
    }
}

impl AsRef<IdentifierPrimitive> for IdentifierExpression {
    fn as_ref(&self) -> &IdentifierPrimitive {
        &self.prim
    }
}

impl AsMut<IdentifierPrimitive> for IdentifierExpression {
    fn as_mut(&mut self) -> &mut IdentifierPrimitive {
        &mut self.prim
    }
}
