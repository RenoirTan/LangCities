use std::any::Any;

use axum::extract::FromRequestParts;
use langcities_lcdcdsl::component::Alias;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::{
    entity::dc_users,
    error::{DcAppError, DcAppErrorTrait},
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, IntoParams)]
pub struct UsersGetQueryDto {
    pub auth_user_id: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[schema(value_type = String)]
pub struct UsersGetParamsDto(pub Alias);

impl<'de> Deserialize<'de> for UsersGetParamsDto {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse::<Alias>()
            .map(Self)
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserDto {
    pub id: i64,
    pub auth_user_id: i64,
}

impl From<dc_users::Model> for UserDto {
    fn from(model: dc_users::Model) -> Self {
        UserDto {
            id: model.id,
            auth_user_id: model.auth_user_id,
        }
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
            .ok_or_else(|| DcAppError::unauthorized(Some(format!("not authorized").into())))
            .cloned()
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
            .map(|m| m.into())
    }
}
