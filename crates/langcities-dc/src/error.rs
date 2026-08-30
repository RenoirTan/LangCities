use std::{error::Error, fmt::Display};

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DcAppErrorKind {
    Database,
    FailedInit,
    Unauthorized,
}

impl Display for DcAppErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct DcAppError {
    pub source: Option<Box<dyn Error + Send + Sync + 'static>>,
    pub kind: DcAppErrorKind,
}

impl DcAppError {
    pub fn new(
        source: Option<Box<dyn Error + Send + Sync + 'static>>,
        kind: DcAppErrorKind,
    ) -> Self {
        Self { source, kind }
    }
}

impl Display for DcAppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DcAppError({:?})", self.kind)
    }
}

impl Error for DcAppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_deref().map(|e| e as &(dyn Error + 'static))
    }
}

pub trait DcAppErrorTrait {
    fn database(source: impl Into<Option<Box<dyn Error + Send + Sync + 'static>>>) -> Self;
    fn failed_init(source: impl Into<Option<Box<dyn Error + Send + Sync + 'static>>>) -> Self;
    fn unauthorized(source: impl Into<Option<Box<dyn Error + Send + Sync + 'static>>>) -> Self;
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
}

impl IntoResponse for DcAppError {
    fn into_response(self) -> Response {
        let status = match self.kind {
            DcAppErrorKind::Unauthorized => StatusCode::UNAUTHORIZED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body = Json(json!({
            "error": &self.source.map(|s| s.to_string()).unwrap_or_else(|| "".into()),
        }));
        (status, body).into_response()
    }
}
