use crate::{
    entity::dc_users,
    error::{DcAppError, DcAppErrorTrait},
    state::AppState,
};
use axum::{RequestPartsExt, extract::FromRequestParts, http::request::Parts};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use langcities_common_server::dto::request::{RequestAccessKind, RequestContext};
use langcities_jwt::payload::Claims;
use langcities_lcdcdsl::component::Id;
use std::ops::{Deref, DerefMut};

#[derive(Clone, Debug)]
pub struct DcClaimsWrapper(pub Claims);

impl DcClaimsWrapper {
    pub fn new(claims: impl Into<Claims>) -> Self {
        Self(claims.into())
    }
}

impl AsRef<Claims> for DcClaimsWrapper {
    fn as_ref(&self) -> &Claims {
        &self
    }
}

impl AsMut<Claims> for DcClaimsWrapper {
    fn as_mut(&mut self) -> &mut Claims {
        &mut *self
    }
}

impl Deref for DcClaimsWrapper {
    type Target = Claims;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DcClaimsWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FromRequestParts<AppState> for DcClaimsWrapper {
    type Rejection = DcAppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(DcAppError::unauthorized)?;

        let token_data = state
            .jwt_decoder
            .decode_token::<()>(&bearer.token())
            .map_err(DcAppError::unauthorized)?;

        Ok(DcClaimsWrapper(token_data.claims))
    }
}

impl FromRequestParts<AppState> for RequestContext {
    type Rejection = DcAppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let user: Option<dc_users::Model> = parts.extract_with_state(state).await?;
        Ok(RequestContext {
            caller_id: user.map(|m| Id::from(m.id)),
            access_kind: RequestAccessKind::NormalUser,
        })
    }
}
