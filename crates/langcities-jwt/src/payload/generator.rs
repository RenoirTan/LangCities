use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::{
    config::JwtConfig,
    error::{JwtError, JwtErrorTrait},
    payload::BaseClaims,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClaimsGenerator {
    pub expiry: Duration,
    pub issuer: String,
}

impl ClaimsGenerator {
    pub fn new<E, I>(expiry: E, issuer: I) -> Self
    where
        E: Into<Duration>,
        I: Into<String>,
    {
        let (expiry, issuer) = (expiry.into(), issuer.into());
        Self { expiry, issuer }
    }

    pub fn from_config(config: &JwtConfig) -> Result<Self, JwtError> {
        let duration =
            Duration::from_std(config.expiry).map_err(|e| JwtError::bad_config(Some(e.into())))?;
        Ok(Self::new(duration, config.issuer.clone()))
    }

    pub fn generate_claims<A>(
        &self,
        audience: impl Into<String>,
        subject: impl Into<String>,
        additional: A,
    ) -> BaseClaims<A> {
        let jti = Uuid::new_v4().to_string();
        let aud = audience.into();
        let iat = Utc::now().timestamp();
        let exp = iat + self.expiry.num_seconds();
        let iss = self.issuer.clone();
        let sub = subject.into();
        BaseClaims {
            jti,
            aud,
            exp,
            iat,
            nbf: iat,
            iss,
            sub,
            add: additional,
        }
    }
}
