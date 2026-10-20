use std::fmt::Display;

use langcities_common::error::LcError;
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

langcities_common::lc_error_trait! {
    MessagingErrorTrait for MessagingError as MessagingErrorKind {
        Serialization, Deserialization,
    }
}
