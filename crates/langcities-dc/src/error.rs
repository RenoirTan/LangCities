use axum::http::StatusCode;
use langcities_common::error::{Error, LcError};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DcAppErrorKind {
    Database,
    FailedInit,
    Unauthorized,
    InvalidAccessToken,
    BadRequest,
    NotFound,
    AuthService,
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
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::AuthService => StatusCode::BAD_GATEWAY,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub type DcAppError = LcError<DcAppErrorKind>;

pub trait DcAppErrorTrait {
    fn database(source: impl Into<Error>) -> Self;
    fn failed_init(source: impl Into<Error>) -> Self;
    fn unauthorized(source: impl Into<Error>) -> Self;
    fn invalid_access_token(source: impl Into<Error>) -> Self;
    fn bad_request(source: impl Into<Error>) -> Self;
    fn not_found(source: impl Into<Error>) -> Self;
    fn auth_service(source: impl Into<Error>) -> Self;
    fn other(source: impl Into<Error>) -> Self;
}

impl DcAppErrorTrait for DcAppError {
    fn database(source: impl Into<Error>) -> Self {
        Self::new(Some(source.into()), DcAppErrorKind::Database)
    }

    fn failed_init(source: impl Into<Error>) -> Self {
        Self::new(Some(source.into()), DcAppErrorKind::FailedInit)
    }

    fn unauthorized(source: impl Into<Error>) -> Self {
        Self::new(Some(source.into()), DcAppErrorKind::Unauthorized)
    }

    fn invalid_access_token(source: impl Into<Error>) -> Self {
        Self::new(Some(source.into()), DcAppErrorKind::InvalidAccessToken)
    }

    fn bad_request(source: impl Into<Error>) -> Self {
        Self::new(Some(source.into()), DcAppErrorKind::BadRequest)
    }

    fn not_found(source: impl Into<Error>) -> Self {
        Self::new(Some(source.into()), DcAppErrorKind::NotFound)
    }

    fn other(source: impl Into<Error>) -> Self {
        Self::new(Some(source.into()), DcAppErrorKind::Other)
    }

    fn auth_service(source: impl Into<Error>) -> Self {
        Self::new(Some(source.into()), DcAppErrorKind::AuthService)
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
            (DcAppErrorKind::BadRequest, StatusCode::BAD_REQUEST),
            (DcAppErrorKind::NotFound, StatusCode::NOT_FOUND),
            (DcAppErrorKind::AuthService, StatusCode::BAD_GATEWAY),
            (DcAppErrorKind::Other, StatusCode::INTERNAL_SERVER_ERROR),
        ];

        for (kind, expected) in cases {
            assert_eq!(Into::<StatusCode>::into(kind), expected);
        }
    }
}
