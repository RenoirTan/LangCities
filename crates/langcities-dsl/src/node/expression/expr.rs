use crate::node::{BinaryExpression, FunctionCall, IdentifierExpression, StringLiteral};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Expr {
    StringLiteral(StringLiteral),
    Identifier(IdentifierExpression),
    Binary(BinaryExpression),
    FunctionCall(FunctionCall),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExpressionNode {
    pub expr: Expr,
}
