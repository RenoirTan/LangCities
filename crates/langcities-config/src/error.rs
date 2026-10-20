use std::fmt::Display;

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

langcities_common::lc_error_trait! {
    LcConfigErrorTrait for LcConfigError as LcConfigErrorKind {
        MissingKey, BadParse, Other,
    }
    declare {
        fn missing_key_of(key: impl Into<String>) -> Self;
    }
    define {
        fn missing_key_of(key: impl Into<String>) -> Self {
            Self::missing_key_desc(format!("Key '{}' missing", key.into()))
        }
    }
}
