use serde::{Deserialize, Serialize};
use utoipa::IntoParams;

use crate::entity::dc_users;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, IntoParams)]
pub struct UsersGetQueryDto {
    pub auth_user_id: i64,
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
