use langcities_lcdcdsl::component::Alias;
use sea_orm::entity::prelude::DateTimeUtc;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::entity::users;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, ToSchema, IntoParams)]
#[into_params(names("alias"))]
#[schema(value_type = String)]
pub struct UserAliasDto(#[param(value_type = String)] pub Alias);

impl<'de> Deserialize<'de> for UserAliasDto {
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct ManyUserAliasDto {
    #[serde(default)]
    #[param(value_type = Vec<String>, style = Form, explode = true)]
    pub aliases: Vec<UserAliasDto>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct UserDto {
    pub id: i64,
    pub username: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTimeUtc,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTimeUtc,
}

impl From<users::Model> for UserDto {
    fn from(value: users::Model) -> Self {
        Self {
            id: value.id,
            username: value.username,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ManyUsersDto {
    #[serde(default)]
    pub users: Vec<UserDto>,
}

impl<U: Into<UserDto>> FromIterator<U> for ManyUsersDto {
    fn from_iter<T: IntoIterator<Item = U>>(iter: T) -> Self {
        let users = iter.into_iter().map(|u| u.into()).collect();
        Self { users }
    }
}
