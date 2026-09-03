use axum::http::StatusCode;
use langcities_common::error::LcError;
use serde::{Deserialize, Serialize};
use std::{error::Error, fmt::Display};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DcAppErrorKind {
    Database,
    FailedInit,
    Unauthorized,
    InvalidAccessToken,
    NotFound,
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
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::NotFound => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub type DcAppError = LcError<DcAppErrorKind>;

pub trait DcAppErrorTrait {
    fn database(source: impl Into<Option<Box<dyn Error + Send + Sync + 'static>>>) -> Self;
    fn failed_init(source: impl Into<Option<Box<dyn Error + Send + Sync + 'static>>>) -> Self;
    fn unauthorized(source: impl Into<Option<Box<dyn Error + Send + Sync + 'static>>>) -> Self;
    fn invalid_access_token(
        source: impl Into<Option<Box<dyn Error + Send + Sync + 'static>>>,
    ) -> Self;
    fn not_found(source: impl Into<Option<Box<dyn Error + Send + Sync + 'static>>>) -> Self;
    fn other(source: impl Into<Option<Box<dyn Error + Send + Sync + 'static>>>) -> Self;
}

impl DcAppErrorTrait for DcAppError {
    fn database(source: impl Into<Option<Box<dyn Error + Send + Sync + 'static>>>) -> Self {
        Self::new(source.into(), DcAppErrorKind::Database)
    }

    fn failed_init(source: impl Into<Option<Box<dyn Error + Send + Sync + 'static>>>) -> Self {
        Self::new(source.into(), DcAppErrorKind::FailedInit)
    }

    fn unauthorized(source: impl Into<Option<Box<dyn Error + Send + Sync + 'static>>>) -> Self {
        Self::new(source.into(), DcAppErrorKind::Unauthorized)
    }

    fn invalid_access_token(
        source: impl Into<Option<Box<dyn Error + Send + Sync + 'static>>>,
    ) -> Self {
        Self::new(source.into(), DcAppErrorKind::InvalidAccessToken)
    }

    fn not_found(source: impl Into<Option<Box<dyn Error + Send + Sync + 'static>>>) -> Self {
        Self::new(source.into(), DcAppErrorKind::NotFound)
    }

    fn other(source: impl Into<Option<Box<dyn Error + Send + Sync + 'static>>>) -> Self {
        Self::new(source.into(), DcAppErrorKind::Other)
    }
}
