#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StringLiteralExpr {
    pub kind: StringLiteralKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StringLiteralKind {
    Unquoted,
    Squoted,
    Dquoted,
}

impl StringLiteralExpr {
    pub fn new(kind: impl Into<StringLiteralKind>) -> Self {
        let kind = kind.into();
        Self { kind }
    }
}
