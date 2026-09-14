use std::{error::Error, sync::Arc};

use langcities_cache::{
    backend::moka::MokaWrapper,
    common::{CacheBackend, Expiry},
};
use langcities_jwt::{
    manager::JwtDecoder,
    microservice::Microservice,
    payload::{ParseJwtClaims, ParsedClaims},
};
use sea_orm::{Database, DatabaseConnection};

use crate::{
    config::Config,
    error::{DcAppError, DcAppErrorTrait},
};

#[derive(Clone, Debug)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: DatabaseConnection,
    pub jwt_decoder: Arc<JwtDecoder>,
    pub username_cache: MokaWrapper<String, i64>,
}

impl AppState {
    pub fn new<C, D, J>(config: C, db: D, jwt_decoder: J) -> Self
    where
        C: Into<Config>,
        D: Into<DatabaseConnection>,
        J: Into<JwtDecoder>,
    {
        let config = config.into();
        let username_cache = MokaWrapper::new(config.dc.username_cache_max_capacity);
        Self {
            config: Arc::new(config),
            db: db.into(),
            jwt_decoder: Arc::new(jwt_decoder.into()),
            username_cache,
        }
    }

    pub async fn set_username_cache(
        &self,
        username: String,
        id: i64,
    ) -> Result<Option<i64>, Box<dyn Error + Send + Sync + 'static>> {
        let ttl = self
            .config
            .dc
            .username_cache_ttl
            .num_milliseconds()
            .try_into()
            .expect("username cache TTL must be non-negative");
        self.username_cache
            .set(username, id, Some(Expiry::Ttl(ttl)))
            .await
    }

    pub async fn create<C>(config: C) -> Result<Self, DcAppError>
    where
        C: Into<Config>,
    {
        let config = config.into();
        let db = Database::connect(config.db.clone().to_connection_options())
            .await
            .map_err(|e| DcAppError::database(Some(e.into())))?;
        let jwt_decoder =
            JwtDecoder::from_config(&config.jwt, Microservice::Dc.allowed_audiences())
                .map_err(|e| DcAppError::failed_init(Some(e.into())))?;
        Ok(Self::new(config, db, jwt_decoder))
    }
}

impl ParseJwtClaims for AppState {
    type Error = DcAppError;

    fn parse_jwt_claims(&self, token: &str) -> ParsedClaims<Self::Error> {
        if token.len() == 0 {
            return ParsedClaims::Missing;
        }
        match self.jwt_decoder.decode_token::<()>(token) {
            Ok(t) => ParsedClaims::Valid(t.claims),
            Err(e) => ParsedClaims::Invalid(DcAppError::unauthorized(Some(e.into()))),
        }
    }

    fn map_err(&self, error: Box<dyn Error + Send + Sync>) -> Self::Error {
        DcAppError::invalid_access_token(Some(error.into()))
    }
}
