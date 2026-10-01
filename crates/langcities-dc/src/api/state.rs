use std::{
    borrow::Borrow,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use langcities_common::error::Error;
use langcities_jwt::{
    manager::JwtDecoder,
    microservice::Microservice,
    payload::{ParseJwtClaims, ParsedClaims},
};

use crate::{
    config::Config,
    error::{DcAppError, DcAppErrorTrait},
    state::AppState,
};

#[derive(Clone, Debug)]
pub struct OuterApiState {
    pub jwt_decoder: Arc<JwtDecoder>,
}

impl OuterApiState {
    pub fn new<J>(jwt_decoder: J) -> Self
    where
        J: Into<JwtDecoder>,
    {
        let jwt_decoder = Arc::new(jwt_decoder.into());
        Self { jwt_decoder }
    }

    pub fn create<C>(config: C) -> Result<Self, DcAppError>
    where
        C: Borrow<Config>,
    {
        let config = config.borrow();
        let jwt_decoder =
            JwtDecoder::from_config(&config.jwt, Microservice::Dc.allowed_audiences())
                .map_err(DcAppError::failed_init)?;
        Ok(Self::new(jwt_decoder))
    }
}

#[derive(Clone, Debug)]
pub struct ApiState {
    pub inner: AppState,
    pub outer: OuterApiState,
}

impl ApiState {
    pub fn new<I, O>(inner: I, outer: O) -> Self
    where
        I: Into<AppState>,
        O: Into<OuterApiState>,
    {
        let (inner, outer) = (inner.into(), outer.into());
        Self { inner, outer }
    }

    pub async fn create_with_state<I, C>(inner: I, config: C) -> Result<Self, DcAppError>
    where
        I: Into<AppState>,
        C: Borrow<Config>,
    {
        let outer = OuterApiState::create(config)?;
        Ok(Self::new(inner, outer))
    }

    pub async fn create<C>(config: C) -> Result<Self, DcAppError>
    where
        C: Into<Config>,
    {
        let config = config.into();
        let outer = OuterApiState::create(&config)?;
        let inner = AppState::create(config).await?;
        Ok(Self::new(inner, outer))
    }

    pub fn jwt_decoder(&self) -> &JwtDecoder {
        &self.outer.jwt_decoder
    }
}

impl Deref for ApiState {
    type Target = AppState;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for ApiState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl ParseJwtClaims for ApiState {
    type Error = DcAppError;

    fn parse_jwt_claims(&self, token: &str) -> ParsedClaims<Self::Error> {
        if token.len() == 0 {
            return ParsedClaims::Missing;
        }
        match self.jwt_decoder().decode_token::<()>(token) {
            Ok(t) => ParsedClaims::Valid(t.claims),
            Err(e) => ParsedClaims::Invalid(DcAppError::unauthorized(e)),
        }
    }

    fn map_err(&self, error: Error) -> Self::Error {
        DcAppError::invalid_access_token(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::{PartialConfig, PartialDcConfig},
        error::DcAppErrorKind,
    };
    use axum::{Json, Router, extract::RawQuery, http::StatusCode, routing::get};
    use langcities_common_db::config::PartialDbConfig;
    use langcities_common_server::config::PartialServerConfig;
    use langcities_jwt::config::{JwtAlgorithm, PartialJwtConfig};
    use sea_orm::DatabaseConnection;
    use serde_json::{Value, json};
    use std::sync::Mutex;

    fn auth_user(id: i64, username: &str) -> Value {
        json!({
            "id": id,
            "username": username,
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-02T00:00:00Z",
        })
    }

    struct MockAuth {
        url: String,
        requests: Arc<Mutex<Vec<String>>>,
        task: tokio::task::JoinHandle<()>,
    }

    impl MockAuth {
        async fn start(status: StatusCode, body: Value) -> Self {
            let requests = Arc::new(Mutex::new(Vec::new()));
            let captured = requests.clone();
            let app = Router::new().route(
                "/v1/users",
                get(move |RawQuery(query): RawQuery| {
                    let captured = captured.clone();
                    let body = body.clone();
                    async move {
                        captured.lock().unwrap().push(query.unwrap_or_default());
                        (status, Json(body))
                    }
                }),
            );
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let url = format!("http://{}", listener.local_addr().unwrap());
            let task = tokio::spawn(async move {
                axum::serve(listener, app).await.unwrap();
            });
            Self {
                url,
                requests,
                task,
            }
        }

        fn state(&self, capacity: u64) -> ApiState {
            let config = Config::from_partial(PartialConfig::new(
                PartialDcConfig {
                    auth_base_url: Some(self.url.clone()),
                    username_cache_max_capacity: Some(capacity),
                    ..Default::default()
                },
                PartialServerConfig::default(),
                PartialDbConfig {
                    url: Some("sqlite::memory:".into()),
                    ..Default::default()
                },
                PartialJwtConfig {
                    hmac_secret: Some("test-secret".into()),
                    algorithm: Some(JwtAlgorithm::HS256),
                    ..Default::default()
                },
            ))
            .unwrap();
            let jwt_decoder =
                JwtDecoder::from_config(&config.jwt, Microservice::Dc.allowed_audiences()).unwrap();
            let outer = OuterApiState::new(jwt_decoder);
            let inner = AppState::new(config, DatabaseConnection::default()).unwrap();
            ApiState::new(inner, outer)
        }
    }

    impl Drop for MockAuth {
        fn drop(&mut self) {
            self.task.abort();
        }
    }

    #[tokio::test]
    async fn batch_requests_use_repeated_aliases_and_cache_all_users() {
        let auth = MockAuth::start(
            StatusCode::OK,
            json!({"users": [
                auth_user(42, "bob"),
                auth_user(17, "alice")
            ]}),
        )
        .await;
        let state = auth.state(100);
        assert!(
            state
                .fetch_and_cache_auth_users(&[])
                .await
                .unwrap()
                .is_empty()
        );
        assert!(auth.requests.lock().unwrap().is_empty());

        state
            .fetch_and_cache_auth_users(&["alice".into(), "bob".into()])
            .await
            .unwrap();
        assert_eq!(
            *auth.requests.lock().unwrap(),
            ["aliases=alice&aliases=bob"]
        );
        assert_eq!(
            state.get_cached_id_from_username(&"alice".into()).await,
            Some(17)
        );
        assert_eq!(
            state.get_cached_id_from_username(&"bob".into()).await,
            Some(42)
        );
        assert_eq!(
            state.resolve_auth_user_id("alice".into()).await.unwrap(),
            17
        );
        assert_eq!(auth.requests.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn cache_miss_fetches_and_matches_username_not_response_order() {
        let auth = MockAuth::start(
            StatusCode::OK,
            json!({"users": [
                auth_user(42, "bob"), auth_user(17, "alice")
            ]}),
        )
        .await;
        let state = auth.state(100);
        assert_eq!(
            state.resolve_auth_user_id("alice".into()).await.unwrap(),
            17
        );
        assert_eq!(*auth.requests.lock().unwrap(), ["aliases=alice"]);
        assert_eq!(
            state.resolve_auth_user_id("alice".into()).await.unwrap(),
            17
        );
        assert_eq!(auth.requests.lock().unwrap().len(), 1);

        // Resolution must also work when caching is disabled by a zero capacity.
        assert_eq!(
            auth.state(0)
                .resolve_auth_user_id("alice".into())
                .await
                .unwrap(),
            17
        );
    }

    #[tokio::test]
    async fn missing_user_is_not_found_and_is_not_cached() {
        let auth = MockAuth::start(StatusCode::OK, json!({"users": []})).await;
        let state = auth.state(100);
        for _ in 0..2 {
            let error = state
                .resolve_auth_user_id("missing".into())
                .await
                .unwrap_err();
            assert_eq!(error.kind, DcAppErrorKind::NotFound);
        }
        assert_eq!(auth.requests.lock().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn upstream_http_and_invalid_response_errors_are_bad_gateway() {
        for (status, body) in [
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"users": [auth_user(17, "alice")]}),
            ),
            (StatusCode::NOT_FOUND, json!({"error": "wrong endpoint"})),
            (StatusCode::OK, json!({"users": "invalid"})),
            (
                StatusCode::OK,
                json!({"users": [{
                    "id": "invalid", "username": "alice",
                    "created_at": "2026-01-01T00:00:00Z",
                    "updated_at": "2026-01-02T00:00:00Z"
                }]}),
            ),
            (
                StatusCode::OK,
                json!({"users": [{"id": 17, "username": "alice"}]}),
            ),
        ] {
            let auth = MockAuth::start(status, body).await;
            let state = auth.state(100);
            let error = state
                .resolve_auth_user_id("alice".into())
                .await
                .unwrap_err();
            assert_eq!(error.kind, DcAppErrorKind::AuthService);
            assert_eq!(
                Into::<StatusCode>::into(error.kind),
                StatusCode::BAD_GATEWAY
            );
            assert_eq!(
                state.get_cached_id_from_username(&"alice".into()).await,
                None
            );
        }
    }

    #[tokio::test]
    async fn transport_errors_are_not_reported_as_missing_users() {
        let auth = MockAuth::start(StatusCode::OK, json!({"users": []})).await;
        let state = auth.state(100);
        auth.task.abort();
        // Await cancellation so the listener has been dropped before requesting.
        while !auth.task.is_finished() {
            tokio::task::yield_now().await;
        }
        let error = state
            .resolve_auth_user_id("alice".into())
            .await
            .unwrap_err();
        assert_eq!(error.kind, DcAppErrorKind::AuthService);
    }

    #[cfg(feature = "sqlite")]
    #[tokio::test]
    async fn username_route_looks_up_dc_user_by_auth_id() {
        use crate::{dto::users::GetUserParamsDto, route::v1::users::get_user_by_alias};
        use axum::extract::{Path, State};
        use sea_orm::{ConnectionTrait, Database};

        let auth =
            MockAuth::start(StatusCode::OK, json!({"users": [auth_user(42, "alice")]})).await;
        let mut state = auth.state(100);
        state.db = Database::connect("sqlite::memory:").await.unwrap();
        state.db.execute_unprepared("CREATE TABLE dc_users (id INTEGER PRIMARY KEY, auth_user_id INTEGER NOT NULL UNIQUE)").await.unwrap();
        state
            .db
            .execute_unprepared("INSERT INTO dc_users (id, auth_user_id) VALUES (7, 42)")
            .await
            .unwrap();

        let Json(user) = get_user_by_alias(
            Path(GetUserParamsDto("alice".parse().unwrap())),
            State(state),
        )
        .await
        .unwrap();
        assert_eq!(user.id, 7);
        assert_eq!(user.auth_user_id, 42);
    }
}
