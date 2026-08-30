use std::{error::Error, fmt::Display};

use langcities_common::error::LcError;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum JwtErrorKind {
    BadConfig,
    Unencodeable,
    Undecodeable,
    Unverifiable,
    Other,
}

impl Display for JwtErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub type JwtError = LcError<JwtErrorKind>;

pub trait JwtErrorTrait {
    fn bad_config(source: impl Into<Option<Box<dyn Error + Send + Sync + 'static>>>) -> Self;
    fn unencodeable(source: impl Into<Option<Box<dyn Error + Send + Sync + 'static>>>) -> Self;
    fn undecodeable(source: impl Into<Option<Box<dyn Error + Send + Sync + 'static>>>) -> Self;
    fn unverifiable(source: impl Into<Option<Box<dyn Error + Send + Sync + 'static>>>) -> Self;
    fn other(source: impl Into<Option<Box<dyn Error + Send + Sync + 'static>>>) -> Self;
}

impl JwtErrorTrait for JwtError {
    fn bad_config(source: impl Into<Option<Box<dyn Error + Send + Sync + 'static>>>) -> Self {
        Self::new(source.into(), JwtErrorKind::BadConfig)
    }

    fn unencodeable(source: impl Into<Option<Box<dyn Error + Send + Sync + 'static>>>) -> Self {
        Self::new(source.into(), JwtErrorKind::Unencodeable)
    }

    fn undecodeable(source: impl Into<Option<Box<dyn Error + Send + Sync + 'static>>>) -> Self {
        Self::new(source.into(), JwtErrorKind::Undecodeable)
    }

    fn unverifiable(source: impl Into<Option<Box<dyn Error + Send + Sync + 'static>>>) -> Self {
        Self::new(source.into(), JwtErrorKind::Unverifiable)
    }

    fn other(source: impl Into<Option<Box<dyn Error + Send + Sync + 'static>>>) -> Self {
        Self::new(source.into(), JwtErrorKind::Other)
    }
}
