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
pub struct LcError<K> {
    pub description: Option<String>,
    pub source: Option<Error>,
    pub kind: K,
}

impl<K> LcError<K> {
    pub fn new<T>(description: Option<String>, source: Option<Error>, kind: T) -> Self
    where
        T: Into<K>,
    {
        let (description, source, kind) = (description.into(), source.into(), kind.into());
        Self {
            description,
            source,
            kind,
        }
    }

    pub fn of_kind(kind: impl Into<K>) -> Self {
        Self::new(None, None, kind)
    }

    pub fn with_desc<D, T>(description: D, kind: T) -> Self
    where
        D: Into<String>,
        T: Into<K>,
    {
        Self::new(Some(description.into()), None, kind)
    }

    pub fn with_source<S, T>(source: S, kind: T) -> Self
    where
        S: Into<Error>,
        T: Into<K>,
    {
        Self::new(None, Some(source.into()), kind)
    }

    pub fn with_all<D, S, T>(description: D, source: S, kind: T) -> Self
    where
        D: Into<String>,
        S: Into<Error>,
        T: Into<K>,
    {
        Self::new(Some(description.into()), Some(source.into()), kind)
    }
}

impl<K> Display for LcError<K>
where
    K: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.description {
            Some(d) => write!(f, "LcError({}: {})", self.kind, d),
            None => write!(f, "LcError({})", self.kind),
        }
    }
}

impl<K> StdError for LcError<K>
where
    K: Debug + Display,
{
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
impl<K> IntoResponse for LcError<K>
where
    K: Debug + Display + Into<StatusCode>,
{
    fn into_response(self) -> Response {
        let error = self.kind.to_string();
        let status = self.kind.into();
        let body = Json(match self.description {
            Some(d) => json!({
                "error": error,
                "description": d,
            }),
            None => json!({
                "error": error,
            }),
        });
        (status, body).into_response()
    }
}

/// Generates the `*ErrorTrait` constructors for every variant of a kind enum.
///
/// Each variant gets `_src`, `_desc` and `_all` helpers.
/// Extra trait items can be added with `declare { ... }` and `define { ... }` blocks.
#[macro_export]
macro_rules! lc_error_trait {
    (@declare { $( $variant:ident ),* $(,)? }) => {
        $crate::pastey::paste! {
            $(
                fn [<$variant:snake _src>](source: impl Into<$crate::error::Error>) -> Self;
                fn [<$variant:snake _desc>](description: impl Into<String>) -> Self;
                fn [<$variant:snake _all>](
                    description: impl Into<String>,
                    source: impl Into<$crate::error::Error>,
                ) -> Self;
            )*
        }
    };

    (@impl $kind:ty { $( $variant:ident ),* $(,)? }) => {
        $crate::pastey::paste! {
            $(
                fn [<$variant:snake _src>](source: impl Into<$crate::error::Error>) -> Self {
                    Self::with_source(source, <$kind>::$variant)
                }
                fn [<$variant:snake _desc>](description: impl Into<String>) -> Self {
                    Self::with_desc(description, <$kind>::$variant)
                }
                fn [<$variant:snake _all>](
                    description: impl Into<String>,
                    source: impl Into<$crate::error::Error>,
                ) -> Self {
                    Self::with_all(description, source, <$kind>::$variant)
                }
            )*
        }
    };

    (
        $trait:ident for $error:ty as $kind:ty {
            $( $variant:ident ),* $(,)?
        }
        $( declare { $( $decl:item )* } )?
        $( define  { $( $def:item  )* } )?
    ) => {
        pub trait $trait {
            $crate::lc_error_trait!(@declare { $($variant),* });
            $( $( $decl )* )?
        }

        impl $trait for $error {
            $crate::lc_error_trait!(@impl $kind { $($variant),* });
            $( $( $def )* )?
        }
    };
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
        let error: LcError<TestErrorKind> = LcError::with_all(
            "public message",
            "sensitive database details",
            TestErrorKind::Internal,
        );
        let response = error.into_response();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&body).unwrap(),
            json!({ "error": "Internal", "description": "public message" })
        );
    }
}
