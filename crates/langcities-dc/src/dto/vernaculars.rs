use sea_orm::entity::prelude::DateTimeUtc;
use serde::{Deserialize, Serialize};
use utoipa::IntoParams;

use crate::entity::vernaculars;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, IntoParams)]
pub struct VernacularsGetQueryDto {
    pub identifier: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VernacularsDto {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub updated_at: DateTimeUtc,
    pub created_at: DateTimeUtc,
    pub owner_id: Option<i64>,
}

impl From<vernaculars::Model> for VernacularsDto {
    fn from(model: vernaculars::Model) -> Self {
        Self {
            id: model.id,
            slug: model.slug,
            name: model.name,
            updated_at: model.updated_at,
            created_at: model.created_at,
            owner_id: model.owner_id,
        }
    }
}
