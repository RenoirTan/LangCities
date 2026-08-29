use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BaseClaims<A> {
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
    /// Additional Data
    pub add: A,
}
