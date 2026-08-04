#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct StringLiteral {
    pub kind: StringLiteralKind,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum StringLiteralKind {
    Unquoted,
    Squoted,
    Dquoted,
}

impl StringLiteral {
    pub fn new(kind: impl Into<StringLiteralKind>) -> Self {
        let kind = kind.into();
        Self { kind }
    }
}
