use std::{error::Error, sync::Arc};

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
}

impl AppState {
    pub fn new<C, D, J>(config: C, db: D, jwt_decoder: J) -> Self
    where
        C: Into<Config>,
        D: Into<DatabaseConnection>,
        J: Into<JwtDecoder>,
    {
        Self {
            config: Arc::new(config.into()),
            db: db.into(),
            jwt_decoder: Arc::new(jwt_decoder.into()),
        }
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
