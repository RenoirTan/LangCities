use std::any::Any;

use axum::{
    Json, RequestPartsExt,
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
    response::IntoResponse,
};
use axum_extra::{TypedHeader, headers::Authorization, headers::authorization::Bearer};
use serde_json::json;

use crate::{
    error::{DcAppError, DcAppErrorTrait},
    state::AppState,
};
use langcities_jwt::payload::Claims;

#[derive(Clone, Debug)]
pub struct DcClaimsWrapper(pub Claims);

impl<S> FromRequestParts<S> for DcClaimsWrapper
where
    S: Any + Send + Sync,
{
    type Rejection = DcAppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = (state as &dyn Any)
            .downcast_ref::<AppState>()
            .expect("AppState not found in router state");

        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| DcAppError::unauthorized(None))?;

        let claims = app_state
            .jwt_decoder
            .decode_token::<()>(&bearer.token())
            .map_err(|e| DcAppError::unauthorized(Some(e.into())))?;

        Ok(DcClaimsWrapper(claims.claims))
    }
}

#[utoipa::path(get, path = "/v1/token/validate")]
#[axum::debug_handler]
pub async fn validate_token(_claims: DcClaimsWrapper) -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "valid": true }))).into_response()
}
