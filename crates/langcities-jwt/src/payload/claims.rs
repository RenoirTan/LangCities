use std::error::Error;

use serde::{Deserialize, Serialize};

use crate::error::{JwtError, JwtErrorTrait};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Claims {
    /// JWT ID
    pub jti: String,
    /// Audience
    pub aud: String,
    /// Expiry
    pub exp: i64,
    /// Issued at
    pub iat: i64,
    /// Issuer
    pub iss: String,
    /// Not valid before
    pub nbf: i64,
    /// Subject
    pub sub: String,
    /// Scope
    pub scope: String,
    /// Resources
    pub resources: Vec<String>,
}

impl Claims {
    pub fn sub_to_id(&self) -> Result<Option<i64>, JwtError> {
        let sub = &self.sub;
        if sub.len() <= 0 {
            Ok(None)
        } else {
            sub.parse::<i64>()
                .map(|id| Some(id))
                .map_err(|e| JwtError::invalid_data(Some(e.into())))
        }
    }
}

pub trait ParseJwtClaims {
    type Error: Error + Send + Sync + 'static;

    fn parse_jwt_claims(&self, token: &str) -> ParsedClaims<Self::Error>;
    fn map_err(&self, error: Box<dyn Error + Send + Sync>) -> Self::Error;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ParsedClaims<E: Error + Send + Sync + 'static> {
    Valid(Claims),
    Invalid(E),
    Missing,
}

impl<E: Error + Send + Sync + 'static> ParsedClaims<E> {
    pub fn from_result(ro: Result<Option<Claims>, E>) -> Self {
        match ro {
            Ok(Some(claims)) => Self::Valid(claims),
            Ok(None) => Self::Missing,
            Err(error) => Self::Invalid(error),
        }
    }

    pub fn to_result(self) -> Result<Option<Claims>, E> {
        match self {
            Self::Valid(claims) => Ok(Some(claims)),
            Self::Invalid(error) => Err(error),
            Self::Missing => Ok(None),
        }
    }
}
