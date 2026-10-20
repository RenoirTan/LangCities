use std::fmt::Display;

use langcities_common::error::LcError;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParserErrorKind {
    InvalidSource,
    BadInitialization,
}

impl Display for ParserErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub type ParserError = LcError<ParserErrorKind>;

langcities_common::lc_error_trait! {
    ParserErrorTrait for ParserError as ParserErrorKind {
        InvalidSource, BadInitialization,
    }
}
