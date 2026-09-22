#[cfg(feature = "axum")]
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
#[cfg(feature = "serde_json")]
use serde_json::json;
use std::{
    error::Error as StdError,
    fmt::{Debug, Display},
};

pub type Error = Box<dyn StdError + Send + Sync + 'static>;

#[derive(Debug)]
pub struct LcError<K: Debug + Display + Send> {
    pub source: Option<Error>,
    pub kind: K,
}

impl<K: Debug + Display + Send> LcError<K> {
    pub fn new(source: Option<Error>, kind: impl Into<K>) -> Self {
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

impl<K: Debug + Display + Send> StdError for LcError<K> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source
            .as_deref()
            .map(|e| e as &(dyn StdError + 'static))
    }

    fn cause(&self) -> Option<&dyn StdError> {
        self.source()
    }
}

#[cfg(feature = "axum")]
impl<K: Debug + Display + Send + Into<StatusCode>> IntoResponse for LcError<K> {
    fn into_response(self) -> Response {
        let error = self.kind.to_string();
        let status = self.kind.into();
        let body = Json(json!({
            "error": error,
        }));
        (status, body).into_response()
    }
}

#[cfg(all(test, feature = "axum"))]
mod tests {
    use super::*;
    use axum::body::to_bytes;

    #[derive(Debug)]
    enum TestErrorKind {
        Internal,
    }

    impl Display for TestErrorKind {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Internal")
        }
    }

    impl From<TestErrorKind> for StatusCode {
        fn from(_: TestErrorKind) -> Self {
            Self::INTERNAL_SERVER_ERROR
        }
    }

    #[tokio::test]
    async fn response_does_not_expose_error_source() {
        let error: LcError<TestErrorKind> = LcError::new(
            Some("sensitive database details".into()),
            TestErrorKind::Internal,
        );
        let response = error.into_response();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&body).unwrap(),
            json!({ "error": "Internal" })
        );
    }
}
