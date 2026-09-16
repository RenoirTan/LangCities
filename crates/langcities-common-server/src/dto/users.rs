use chrono::{DateTime, Utc};
use langcities_lcdcdsl::component::Alias;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema, IntoParams)]
#[into_params(names("alias"))]
#[schema(value_type = String)]
pub struct UserAliasDto(#[param(value_type = String)] pub Alias);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct ManyUserAliasDto {
    #[serde(default)]
    #[param(value_type = Vec<String>, style = Form, explode = true)]
    pub aliases: Vec<UserAliasDto>,
}

/// The public user response returned by the auth service.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct AuthUserDto {
    pub id: i64,
    pub username: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct AuthUsersDto {
    #[serde(default)]
    pub users: Vec<AuthUserDto>,
}

impl<U: Into<AuthUserDto>> FromIterator<U> for AuthUsersDto {
    fn from_iter<T: IntoIterator<Item = U>>(iter: T) -> Self {
        let users = iter.into_iter().map(|u| u.into()).collect();
        Self { users }
    }
}
