use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct AccessTokenResponseDto {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

impl AccessTokenResponseDto {
    pub fn new<A, T, E>(access_token: A, token_type: T, expires_in: E) -> Self
    where
        A: Into<String>,
        T: Into<String>,
        E: Into<i64>,
    {
        let (access_token, token_type, expires_in) =
            (access_token.into(), token_type.into(), expires_in.into());
        Self {
            access_token,
            token_type,
            expires_in,
        }
    }
}
