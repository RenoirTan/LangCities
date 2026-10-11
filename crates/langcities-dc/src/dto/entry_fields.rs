use std::fmt::Display;

use chrono::{DateTime, Utc};
use langcities_lcdcdsl::component::{EntryAlias, EntryFieldAlias, Id, UserAlias, VernacularAlias};
use sea_orm::ActiveValue;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    dto::entries::EntryAliasDto,
    entity::{entries, entry_fields},
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[schema(value_type = String)]
pub struct EntryFieldAliasDto(pub EntryFieldAlias);

impl Display for EntryFieldAliasDto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct EntryFieldDto {
    pub id: i64,
    pub entry_id: i64,
    pub slug: String,
    pub override_value: String,
    pub dirty_value: String,
    pub clean_value: String,
    pub updated_at: DateTime<Utc>,
}

impl From<entry_fields::Model> for EntryFieldDto {
    fn from(value: entry_fields::Model) -> Self {
        Self {
            id: value.id,
            entry_id: value.entry_id,
            slug: value.slug,
            override_value: value.override_value,
            dirty_value: value.dirty_value,
            clean_value: value.clean_value,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CreateEntryFieldDto {
    pub entry: EntryAliasDto,
    pub slug: String,
    pub override_value: Option<String>,
}

impl CreateEntryFieldDto {
    pub fn to_active_model(self, entry: &entries::Model) -> entry_fields::ActiveModel {
        let mut am = entry_fields::ActiveModel {
            dirty_value: ActiveValue::Set("".into()),
            clean_value: ActiveValue::Set("".into()),
            ..Default::default()
        };
        self.update_active_model(&mut am, entry);
        am
    }

    pub fn update_active_model<'m>(
        self,
        am: &'m mut entry_fields::ActiveModel,
        entry: &entries::Model,
    ) -> &'m mut entry_fields::ActiveModel {
        am.entry_id = ActiveValue::Set(entry.id);
        am.slug = ActiveValue::Set(self.slug);
        am.override_value = ActiveValue::Set(self.override_value.unwrap_or_default());
        am
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct UpdateEntryFieldDto {
    pub override_value: Option<String>,
}

impl UpdateEntryFieldDto {
    pub fn to_active_model(self) -> entry_fields::ActiveModel {
        let mut am = entry_fields::ActiveModel::default();
        self.update_active_model(&mut am);
        am
    }

    pub fn update_active_model<'m>(
        self,
        am: &'m mut entry_fields::ActiveModel,
    ) -> &'m mut entry_fields::ActiveModel {
        if let Some(o) = self.override_value {
            am.override_value = ActiveValue::Set(o);
        }
        am.updated_at = ActiveValue::Set(Utc::now());
        am
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct EntryFieldWithContext {
    pub entry_field: EntryFieldDto,
    pub entry_index: i64,
    pub vernacular_id: i64,
    pub vernacular_slug: String,
    pub owner_auth_user_id: i64,
    /// username may not be populated if not specifically called for
    pub owner_username: Option<String>,
}

impl EntryFieldWithContext {
    pub fn matches_alias(&self, alias: &EntryFieldAlias, caller_id: &Id) -> bool {
        let ef_alias = match alias {
            EntryFieldAlias::Id(id) => return **id == self.entry_field.id,
            EntryFieldAlias::Alias(alias) => alias,
        };
        if *ef_alias.slug != self.entry_field.slug {
            return false;
        }
        let entry_alias = match &ef_alias.entry_alias {
            EntryAlias::Id(id) => return **id == self.entry_field.entry_id,
            EntryAlias::Alias(alias) => alias,
        };
        if *entry_alias.index != self.entry_index {
            return false;
        }
        let ver_alias = match &entry_alias.vernacular_alias {
            VernacularAlias::Id(id) => return **id == self.vernacular_id,
            VernacularAlias::Slug(slug) => {
                return **slug == self.vernacular_slug && **caller_id == self.owner_auth_user_id;
            }
            VernacularAlias::Alias(alias) => alias,
        };
        if *ver_alias.slug != self.vernacular_slug {
            return false;
        }
        match &ver_alias.user_alias {
            UserAlias::Id(id) => **id == self.owner_auth_user_id,
            UserAlias::Slug(username) => Some(&**username) == self.owner_username.as_ref(),
        }
    }
}
