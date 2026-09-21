use std::fmt::Display;

use langcities_common::error::{Error, LcError};

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

pub trait ParserErrorTrait {
    fn invalid_source(source: impl Into<Error>) -> Self;
    fn bad_initialization(source: impl Into<Error>) -> Self;
}

impl ParserErrorTrait for ParserError {
    fn invalid_source(source: impl Into<Error>) -> Self {
        Self::new(Some(source.into()), ParserErrorKind::InvalidSource)
    }

    fn bad_initialization(source: impl Into<Error>) -> Self {
        Self::new(Some(source.into()), ParserErrorKind::BadInitialization)
    }
}
