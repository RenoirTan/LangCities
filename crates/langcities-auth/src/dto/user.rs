use sea_orm::entity::prelude::DateTimeUtc;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::entity::users;

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
