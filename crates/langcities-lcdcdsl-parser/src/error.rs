use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub struct ParserError {
    pub source: Option<Box<dyn Error>>,
    pub kind: ParserErrorKind,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParserErrorKind {
    InvalidSource,
    BadInitialization,
}

impl ParserError {
    pub fn new(source: Option<Box<dyn Error>>, kind: impl Into<ParserErrorKind>) -> Self {
        let kind = kind.into();
        Self { source, kind }
    }
}

impl Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ParserError({})", self.kind)
    }
}

impl Display for ParserErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for ParserError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_ref().map(|e| e.as_ref())
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}
