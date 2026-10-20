use std::fmt::Display;

use axum::http::StatusCode;
use langcities_common::error::LcError;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthAppErrorKind {
    Database,
    InvalidCredentials,
    Conflict,
    PasswordHashing,
    FailedInit,
    Other,
    FailedSession,
    TokenGeneration,
    Unauthorized,
    NotFound,
}

impl Display for AuthAppErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Into<StatusCode> for AuthAppErrorKind {
    fn into(self) -> StatusCode {
        match self {
            Self::InvalidCredentials => StatusCode::UNAUTHORIZED,
            Self::Conflict => StatusCode::CONFLICT,
            Self::Database => StatusCode::INTERNAL_SERVER_ERROR,
            Self::PasswordHashing => StatusCode::INTERNAL_SERVER_ERROR,
            Self::FailedInit => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Other => StatusCode::INTERNAL_SERVER_ERROR,
            Self::FailedSession => StatusCode::INTERNAL_SERVER_ERROR,
            Self::TokenGeneration => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::NotFound => StatusCode::NOT_FOUND,
        }
    }
}

pub type AuthAppError = LcError<AuthAppErrorKind>;

langcities_common::lc_error_trait! {
    AuthAppErrorTrait for AuthAppError as AuthAppErrorKind {
        Database, InvalidCredentials, Conflict, PasswordHashing, FailedInit,
        Other, FailedSession, TokenGeneration, Unauthorized, NotFound,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_kinds_map_to_expected_http_statuses() {
        let cases = [
            (
                AuthAppErrorKind::Database,
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            (
                AuthAppErrorKind::InvalidCredentials,
                StatusCode::UNAUTHORIZED,
            ),
            (AuthAppErrorKind::Conflict, StatusCode::CONFLICT),
            (
                AuthAppErrorKind::PasswordHashing,
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            (
                AuthAppErrorKind::FailedInit,
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            (AuthAppErrorKind::Other, StatusCode::INTERNAL_SERVER_ERROR),
            (
                AuthAppErrorKind::FailedSession,
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            (
                AuthAppErrorKind::TokenGeneration,
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            (AuthAppErrorKind::Unauthorized, StatusCode::UNAUTHORIZED),
            (AuthAppErrorKind::NotFound, StatusCode::NOT_FOUND),
        ];

        for (kind, expected) in cases {
            assert_eq!(Into::<StatusCode>::into(kind), expected);
        }
    }
}
