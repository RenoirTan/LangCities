use std::any::Any;

use axum::{RequestPartsExt, extract::FromRequestParts, http::request::Parts};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use langcities_jwt::payload::Claims;

use crate::{
    error::{DcAppError, DcAppErrorTrait},
    state::AppState,
};

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

        let token_data = app_state
            .jwt_decoder
            .decode_token::<()>(&bearer.token())
            .map_err(|e| DcAppError::unauthorized(Some(e.into())))?;

        Ok(DcClaimsWrapper(token_data.claims))
    }
}
