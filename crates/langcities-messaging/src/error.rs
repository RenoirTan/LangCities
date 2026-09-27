use std::fmt::Display;

use langcities_common::error::{Error, LcError};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessagingErrorKind {
    Serialization,
    Deserialization,
}

impl Display for MessagingErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub type MessagingError = LcError<MessagingErrorKind>;

pub trait MessagingErrorTrait {
    fn serialization(source: impl Into<Error>) -> Self;
    fn deserialization(source: impl Into<Error>) -> Self;
}

impl MessagingErrorTrait for MessagingError {
    fn serialization(source: impl Into<Error>) -> Self {
        Self::new(Some(source.into()), MessagingErrorKind::Serialization)
    }

    fn deserialization(source: impl Into<Error>) -> Self {
        Self::new(Some(source.into()), MessagingErrorKind::Deserialization)
    }
}
