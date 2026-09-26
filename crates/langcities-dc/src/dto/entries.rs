use std::fmt::Display;

use chrono::{DateTime, Utc};
use langcities_lcdcdsl::component::EntryAlias;
use sea_orm::ActiveValue;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    dto::vernaculars::VernacularAliasDto,
    entity::{entries, vernaculars},
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[schema(value_type = String)]
pub struct EntryAliasDto(pub EntryAlias);

impl Display for EntryAliasDto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum EntryAction {
    Read,
    Write,
    Delete,
}

impl EntryAction {
    pub fn enforce_owner(&self) -> bool {
        !matches!(self, Self::Read)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct EntryDto {
    pub id: i64,
    pub vernacular_id: i64,
    pub index: i64,
    pub created_at: DateTime<Utc>,
}

impl From<entries::Model> for EntryDto {
    fn from(value: entries::Model) -> Self {
        Self {
            id: value.id,
            vernacular_id: value.vernacular_id,
            index: value.index,
            created_at: value.created_at,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CreateEntryDto {
    pub vernacular: VernacularAliasDto,
}

impl CreateEntryDto {
    pub fn to_active_model(self, vernacular: &vernaculars::Model) -> entries::ActiveModel {
        entries::ActiveModel {
            vernacular_id: ActiveValue::Set(vernacular.id),
            index: ActiveValue::Set(vernacular.next_entry_id),
            ..Default::default()
        }
    }
}
