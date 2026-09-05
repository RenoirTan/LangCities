use std::sync::Arc;

use axum::{
    RequestExt, RequestPartsExt,
    extract::{FromRequestParts, Request, State},
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
    typed_header::TypedHeaderRejectionReason,
};

use crate::payload::{Claims, ParseJwtClaims, ParsedClaims};

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
        let parsed_claims = parts
            .extensions
            .get::<Arc<ParsedClaims<S::Error>>>()
            .ok_or_else(|| state.map_err("ParsedClaims missing from request extensions".into()))?;

        match parsed_claims.as_ref() {
            ParsedClaims::Valid(claims) => Ok(claims.clone()),
            ParsedClaims::Invalid(_) => Err(state.map_err("request claims were invalid".into())),
            ParsedClaims::Missing => Err(state.map_err("request claims were missing".into())),
        }
    }
}

pub async fn parse_token_and_extend_state<S>(
    State(state): State<S>,
    mut req: Request,
    next: Next,
) -> Response
where
    S: Send + Sync + ParseJwtClaims,
    S::Error: IntoResponse,
{
    let parsed_claims = match req
        .extract_parts_with_state::<ParsedClaims<S::Error>, _>(&state)
        .await
    {
        Ok(parsed_claims) => parsed_claims,
        Err(error) => return error.into_response(),
    };
    req.extensions_mut().insert(Arc::new(parsed_claims));
    next.run(req).await
}

impl<S> FromRequestParts<S> for ParsedClaims<S::Error>
where
    S: Send + Sync + ParseJwtClaims,
    S::Error: IntoResponse,
{
    type Rejection = S::Error;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let bearer = match parts.extract::<TypedHeader<Authorization<Bearer>>>().await {
            Ok(TypedHeader(Authorization(bearer))) => bearer,
            Err(e) => {
                return Ok(match e.reason() {
                    TypedHeaderRejectionReason::Missing => Self::Missing,
                    TypedHeaderRejectionReason::Error(_) => Self::Invalid(state.map_err(e.into())),
                    _ => Self::Invalid(state.map_err("other".into())),
                });
            }
        };

        Ok(state.parse_jwt_claims(&bearer.token()))
    }
}
