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
