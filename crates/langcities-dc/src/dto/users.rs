use std::any::Any;

use axum::extract::FromRequestParts;
use langcities_lcdcdsl::component::{Id, UserAlias};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::{
    api::state::ApiState,
    entity::dc_users,
    error::{DcAppError, DcAppErrorTrait},
    state::AppState,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, IntoParams)]
pub struct GetUsersQueryDto {
    pub auth_user_id: Id,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[schema(value_type = String)]
pub struct GetUserParamsDto(pub UserAlias);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct UserDto {
    pub id: Id,
    pub auth_user_id: Id,
}

impl TryFrom<dc_users::Model> for UserDto {
    type Error = DcAppError;

    fn try_from(model: dc_users::Model) -> Result<Self, Self::Error> {
        Ok(UserDto {
            id: model.id.into(),
            auth_user_id: model.auth_user_id.into(),
        })
    }
}

impl<S> FromRequestParts<S> for dc_users::Model
where
    S: Any + Send + Sync,
{
    type Rejection = DcAppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<dc_users::Model>()
            .ok_or_else(|| DcAppError::unauthorized(format!("not authorized")))
            .cloned()
    }
}

impl FromRequestParts<ApiState> for Option<dc_users::Model> {
    type Rejection = DcAppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &ApiState,
    ) -> Result<Self, Self::Rejection> {
        Ok(dc_users::Model::from_request_parts(parts, state).await.ok())
    }
}

impl<S> FromRequestParts<S> for UserDto
where
    S: Any + Send + Sync,
{
    type Rejection = DcAppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        dc_users::Model::from_request_parts(parts, state)
            .await
            .map(|m| m.try_into())
            .flatten()
    }
}

impl FromRequestParts<AppState> for Option<UserDto> {
    type Rejection = DcAppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(UserDto::from_request_parts(parts, state).await.ok())
    }
}
