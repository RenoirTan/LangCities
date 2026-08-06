use super::{
    BinaryExpr, FunctionCallExpr, IdentifierExpr, IdentifierPrim, MultiExpr, StringLiteralExpr,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NodeKind {
    MultiExpr(MultiExpr),
    IdentifierExpr(IdentifierExpr),
    IdentifierPrim(IdentifierPrim),
    StringLiteralExpr(StringLiteralExpr),
    BinaryExpr(BinaryExpr),
    FunctionCallExpr(FunctionCallExpr),
}
