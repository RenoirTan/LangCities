use std::sync::Arc;

use langcities_jwt::manager::JwtEncoder;
use langcities_jwt::payload::ClaimsGenerator;
use sea_orm::{Database, DatabaseConnection};

use crate::{
    config::Config,
    error::{AuthAppError, AuthAppErrorTrait},
    util::password::PasswordChecker,
};

#[derive(Clone, Debug)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: DatabaseConnection,
    pub pw_checker: PasswordChecker,
    pub jwt_encoder: Arc<JwtEncoder>,
    pub claims_generator: Arc<ClaimsGenerator>,
}

impl AppState {
    pub fn new<C, D, P, E, G>(
        config: C,
        db: D,
        pw_checker: P,
        jwt_encoder: E,
        claims_generator: G,
    ) -> Self
    where
        C: Into<Config>,
        D: Into<DatabaseConnection>,
        P: Into<PasswordChecker>,
        E: Into<JwtEncoder>,
        G: Into<ClaimsGenerator>,
    {
        Self {
            config: Arc::new(config.into()),
            db: db.into(),
            pw_checker: pw_checker.into(),
            jwt_encoder: Arc::new(jwt_encoder.into()),
            claims_generator: Arc::new(claims_generator.into()),
        }
    }

    pub async fn create<C>(config: C) -> Result<Self, AuthAppError>
    where
        C: Into<Config>,
    {
        let config = config.into();
        let db = Database::connect(config.db.clone().to_connection_options())
            .await
            .map_err(|e| AuthAppError::database(Some(e.into())))?;
        let pw_checker = PasswordChecker::default();
        let jwt_encoder = JwtEncoder::from_config(&config.jwt)
            .map_err(|e| AuthAppError::failed_init(Some(e.into())))?;
        let claims_generator = ClaimsGenerator::from_config(&config.jwt)
            .map_err(|e| AuthAppError::failed_init(Some(e.into())))?;
        Ok(Self::new(
            config,
            db,
            pw_checker,
            jwt_encoder,
            claims_generator,
        ))
    }
}
