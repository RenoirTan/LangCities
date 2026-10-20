use axum::http::StatusCode;
use langcities_common::error::LcError;
use sea_orm::TransactionError;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DcAppErrorKind {
    Database,
    FailedInit,
    Unauthorized,
    InvalidAccessToken,
    Conflict,
    /// When server (potentially) causes a invalid state
    BadState,
    BadRequest,
    NotFound,
    AuthService,
    Cache,
    /// Cannot send or receive any messages
    Messaging,
    Unparseable,
    Other,
}

impl Display for DcAppErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Into<StatusCode> for DcAppErrorKind {
    fn into(self) -> StatusCode {
        match self {
            Self::Unauthorized | Self::InvalidAccessToken => StatusCode::UNAUTHORIZED,
            Self::BadRequest => StatusCode::BAD_REQUEST,
            Self::Conflict => StatusCode::CONFLICT,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::AuthService => StatusCode::BAD_GATEWAY,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub type DcAppError = LcError<DcAppErrorKind>;

langcities_common::lc_error_trait! {
    DcAppErrorTrait for DcAppError as DcAppErrorKind {
        Database, FailedInit, Unauthorized, InvalidAccessToken, Conflict,
        BadState, BadRequest, NotFound, AuthService, Cache, Messaging,
        Unparseable, Other,
    }
    declare {
        fn flatten_tx(e: TransactionError<DcAppError>) -> DcAppError;
    }
    define {
        fn flatten_tx(e: TransactionError<DcAppError>) -> DcAppError {
            match e {
                TransactionError::Connection(e) => DcAppError::database_src(e),
                TransactionError::Transaction(e) => e,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_kinds_map_to_expected_http_statuses() {
        let cases = [
            (DcAppErrorKind::Database, StatusCode::INTERNAL_SERVER_ERROR),
            (
                DcAppErrorKind::FailedInit,
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            (DcAppErrorKind::Unauthorized, StatusCode::UNAUTHORIZED),
            (DcAppErrorKind::InvalidAccessToken, StatusCode::UNAUTHORIZED),
            (DcAppErrorKind::Conflict, StatusCode::CONFLICT),
            (DcAppErrorKind::BadRequest, StatusCode::BAD_REQUEST),
            (DcAppErrorKind::NotFound, StatusCode::NOT_FOUND),
            (DcAppErrorKind::AuthService, StatusCode::BAD_GATEWAY),
            (DcAppErrorKind::Cache, StatusCode::INTERNAL_SERVER_ERROR),
            (DcAppErrorKind::Other, StatusCode::INTERNAL_SERVER_ERROR),
        ];

        for (kind, expected) in cases {
            assert_eq!(Into::<StatusCode>::into(kind), expected);
        }
    }
}
