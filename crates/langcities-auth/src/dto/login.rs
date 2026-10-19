use langcities_lcdcdsl::component::Slug;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct PasswordLoginDto {
    pub username: Slug,
    pub password: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct PasswordLoginResponseDto {
    pub message: String,
}
