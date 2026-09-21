use std::fmt::Display;

use langcities_common::error::{Error, LcError};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum JwtErrorKind {
    BadConfig,
    Unencodeable,
    Undecodeable,
    Unverifiable,
    InvalidData,
    Other,
}

impl Display for JwtErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub type JwtError = LcError<JwtErrorKind>;

pub trait JwtErrorTrait {
    fn bad_config(source: impl Into<Error>) -> Self;
    fn unencodeable(source: impl Into<Error>) -> Self;
    fn undecodeable(source: impl Into<Error>) -> Self;
    fn unverifiable(source: impl Into<Error>) -> Self;
    fn invalid_data(source: impl Into<Error>) -> Self;
    fn other(source: impl Into<Error>) -> Self;
}

impl JwtErrorTrait for JwtError {
    fn bad_config(source: impl Into<Error>) -> Self {
        Self::new(Some(source.into()), JwtErrorKind::BadConfig)
    }

    fn unencodeable(source: impl Into<Error>) -> Self {
        Self::new(Some(source.into()), JwtErrorKind::Unencodeable)
    }

    fn undecodeable(source: impl Into<Error>) -> Self {
        Self::new(Some(source.into()), JwtErrorKind::Undecodeable)
    }

    fn unverifiable(source: impl Into<Error>) -> Self {
        Self::new(Some(source.into()), JwtErrorKind::Unverifiable)
    }

    fn invalid_data(source: impl Into<Error>) -> Self {
        Self::new(Some(source.into()), JwtErrorKind::InvalidData)
    }

    fn other(source: impl Into<Error>) -> Self {
        Self::new(Some(source.into()), JwtErrorKind::Other)
    }
}
