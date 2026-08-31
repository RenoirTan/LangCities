use std::{error::Error, fmt::Display};

use langcities_common::error::LcError;

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
    fn missing_key(key: impl Into<String>) -> Self;

    fn bad_parse(source: Option<Box<dyn Error + Send + Sync + 'static>>) -> Self;

    fn other(source: Option<Box<dyn Error + Send + Sync + 'static>>) -> Self;
}

impl LcConfigErrorTrait for LcConfigError {
    fn missing_key(key: impl Into<String>) -> Self {
        let msg = format!("Key '{}' missing", key.into());
        Self::new(Some(msg.into()), LcConfigErrorKind::MissingKey)
    }

    fn bad_parse(source: Option<Box<dyn Error + Send + Sync + 'static>>) -> Self {
        Self::new(source.into(), LcConfigErrorKind::BadParse)
    }

    fn other(source: Option<Box<dyn Error + Send + Sync + 'static>>) -> Self {
        Self::new(source.into(), LcConfigErrorKind::Other)
    }
}
