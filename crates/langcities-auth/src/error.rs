use std::{error::Error, fmt::Display};

use axum::{Json, http::StatusCode, response::IntoResponse};
use langcities_common::error::LcError;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthAppErrorKind {
    Database,
    InvalidCredentials,
    PasswordHashing,
    FailedInit,
    Other,
    FailedSession,
}

impl Display for AuthAppErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub type AuthAppError = LcError<AuthAppErrorKind>;

pub trait AuthAppErrorTrait {
    fn invalid_credentials(
        source: impl Into<Option<Box<dyn Error + Send + Sync + 'static>>>,
    ) -> Self;
    fn database(source: impl Into<Option<Box<dyn Error + Send + Sync + 'static>>>) -> Self;
    fn password_hashing(source: impl Into<Option<Box<dyn Error + Send + Sync + 'static>>>) -> Self;
    fn failed_init(source: impl Into<Option<Box<dyn Error + Send + Sync + 'static>>>) -> Self;
    fn failed_session(source: impl Into<Option<Box<dyn Error + Send + Sync + 'static>>>) -> Self;
    fn other(source: impl Into<Option<Box<dyn Error + Send + Sync + 'static>>>) -> Self;

    fn to_response(self) -> AuthAppErrorResponseDto;
}

impl AuthAppErrorTrait for AuthAppError {
    fn invalid_credentials(
        source: impl Into<Option<Box<dyn Error + Send + Sync + 'static>>>,
    ) -> Self {
        Self::new(source.into(), AuthAppErrorKind::InvalidCredentials)
    }

    fn database(source: impl Into<Option<Box<dyn Error + Send + Sync + 'static>>>) -> Self {
        Self::new(source.into(), AuthAppErrorKind::Database)
    }

    fn password_hashing(source: impl Into<Option<Box<dyn Error + Send + Sync + 'static>>>) -> Self {
        Self::new(source.into(), AuthAppErrorKind::PasswordHashing)
    }

    fn failed_init(source: impl Into<Option<Box<dyn Error + Send + Sync + 'static>>>) -> Self {
        Self::new(source.into(), AuthAppErrorKind::FailedInit)
    }

    fn failed_session(source: impl Into<Option<Box<dyn Error + Send + Sync + 'static>>>) -> Self {
        Self::new(source.into(), AuthAppErrorKind::FailedSession)
    }

    fn other(source: impl Into<Option<Box<dyn Error + Send + Sync + 'static>>>) -> Self {
        Self::new(source.into(), AuthAppErrorKind::Other)
    }

    fn to_response(self) -> AuthAppErrorResponseDto {
        let error = match self.source() {
            Some(s) => format!("{}", s),
            None => "An error occurred".into(),
        };
        let kind = self.kind;
        AuthAppErrorResponseDto { error, kind }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthAppErrorResponseDto {
    pub error: String,
    pub kind: AuthAppErrorKind,
}

impl IntoResponse for AuthAppErrorResponseDto {
    fn into_response(self) -> axum::response::Response {
        let status = match self.kind {
            AuthAppErrorKind::InvalidCredentials => StatusCode::BAD_REQUEST,
            AuthAppErrorKind::Database => StatusCode::INTERNAL_SERVER_ERROR,
            AuthAppErrorKind::PasswordHashing => StatusCode::INTERNAL_SERVER_ERROR,
            AuthAppErrorKind::FailedInit => StatusCode::INTERNAL_SERVER_ERROR,
            AuthAppErrorKind::Other => StatusCode::INTERNAL_SERVER_ERROR,
            AuthAppErrorKind::FailedSession => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body = Json(json!({
            "error": self.error,
        }));
        (status, body).into_response()
    }
}
