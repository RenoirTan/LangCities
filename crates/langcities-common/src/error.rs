#[cfg(feature = "axum")]
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
#[cfg(feature = "serde_json")]
use serde_json::json;
use std::{
    error::Error,
    fmt::{Debug, Display},
};

#[derive(Debug)]
pub struct LcError<K: Debug + Display + Send> {
    pub source: Option<Box<dyn Error + Send + Sync + 'static>>,
    pub kind: K,
}

impl<K: Debug + Display + Send> LcError<K> {
    pub fn new(source: Option<Box<dyn Error + Send + Sync + 'static>>, kind: impl Into<K>) -> Self {
        let kind = kind.into();
        Self { source, kind }
    }
}

impl<K: Debug + Display + Send> Display for LcError<K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.source {
            Some(s) => write!(f, "LcError({}, {})", self.kind, s),
            None => write!(f, "LcError({})", self.kind),
        }
    }
}

impl<K: Debug + Display + Send> Error for LcError<K> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_deref().map(|e| e as &(dyn Error + 'static))
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}

#[cfg(feature = "axum")]
impl<K: Debug + Display + Send + Into<StatusCode>> IntoResponse for LcError<K> {
    fn into_response(self) -> Response {
        let body = Json(json!({
            "error": format!("{}", self),
        }));
        let status = self.kind.into();
        (status, body).into_response()
    }
}
