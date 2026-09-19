use crate::{
    error::{DcAppError, DcAppErrorTrait},
    state::AppState,
};
use axum::{RequestPartsExt, extract::FromRequestParts, http::request::Parts};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use langcities_common_server::dto::request::{RequestAccessKind, RequestContext};
use langcities_jwt::payload::{Claims, ParsedClaims};
use langcities_lcdcdsl::component::Id;
use std::{
    any::Any,
    ops::{Deref, DerefMut},
};

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
            .map_err(|r| DcAppError::unauthorized(Some(r.into())))?;

        let token_data = app_state
            .jwt_decoder
            .decode_token::<()>(&bearer.token())
            .map_err(|e| DcAppError::unauthorized(Some(e.into())))?;

        Ok(DcClaimsWrapper(token_data.claims))
    }
}

impl FromRequestParts<AppState> for RequestContext {
    type Rejection = DcAppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let parsed_claims: ParsedClaims<DcAppError> = parts.extract_with_state(state).await?;
        match parsed_claims {
            ParsedClaims::Valid(claims) => Ok(RequestContext {
                caller_id: claims
                    .sub_to_id()
                    .map(|id| id.map(Id::from))
                    .map_err(|e| DcAppError::invalid_access_token(Some(e.into())))?,
                access_kind: RequestAccessKind::NormalUser,
            }),
            ParsedClaims::Invalid(e) => Err(e),
            ParsedClaims::Missing => Ok(RequestContext {
                caller_id: None,
                access_kind: RequestAccessKind::NormalUser,
            }),
        }
    }
}
