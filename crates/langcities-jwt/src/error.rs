use std::fmt::Display;

use langcities_common::error::LcError;

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

langcities_common::lc_error_trait! {
    JwtErrorTrait for JwtError as JwtErrorKind {
        BadConfig, Unencodeable, Undecodeable, Unverifiable, InvalidData, Other,
    }
}
