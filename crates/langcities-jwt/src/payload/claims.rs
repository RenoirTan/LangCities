use serde::{Deserialize, Serialize};

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
