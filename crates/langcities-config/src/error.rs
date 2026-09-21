use std::fmt::Display;

use langcities_common::error::{Error, LcError};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LcConfigErrorKind {
    MissingKey,
    BadParse,
    Other,
}

impl Display for LcConfigErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub type LcConfigError = LcError<LcConfigErrorKind>;

pub trait LcConfigErrorTrait {
    fn missing_key_of(key: impl Into<String>) -> Self;

    fn missing_key(source: impl Into<Error>) -> Self;

    fn bad_parse(source: impl Into<Error>) -> Self;

    fn other(source: impl Into<Error>) -> Self;
}

impl LcConfigErrorTrait for LcConfigError {
    fn missing_key_of(key: impl Into<String>) -> Self {
        let msg = format!("Key '{}' missing", key.into());
        Self::missing_key(msg)
    }

    fn missing_key(source: impl Into<Error>) -> Self {
        Self::new(Some(source.into()), LcConfigErrorKind::MissingKey)
    }

    fn bad_parse(source: impl Into<Error>) -> Self {
        Self::new(Some(source.into()), LcConfigErrorKind::BadParse)
    }

    fn other(source: impl Into<Error>) -> Self {
        Self::new(Some(source.into()), LcConfigErrorKind::Other)
    }
}
