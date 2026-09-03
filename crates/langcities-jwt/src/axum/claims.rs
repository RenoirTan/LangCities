use axum::{RequestPartsExt, extract::FromRequestParts, response::IntoResponse};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};

use crate::payload::{Claims, ParseJwtClaims};

impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync + ParseJwtClaims,
    S::Error: IntoResponse,
{
    type Rejection = S::Error;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|e| state.map_err(e.into()))?;

        state.parse_jwt_claims(&bearer.token())
    }
}
