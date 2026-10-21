use std::{collections::HashMap, fmt::Display};

use chrono::{DateTime, Utc};
use langcities_lcdcdsl::component::{
    EntryFieldAlias, EntryFieldRef, EntryRef, Id, Index, Slug, UserAlias, VernacularRef,
};
use sea_orm::ActiveValue;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::{
    dto::entries::EntryAliasDto,
    entity::{entries, entry_fields},
    error::{DcAppError, DcAppErrorTrait},
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[schema(value_type = String)]
pub struct EntryFieldAliasDto(pub EntryFieldAlias);

impl Display for EntryFieldAliasDto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// TODO: optional current_entry_id
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct GetManyEntryFieldAliasDto {
    pub aliases: Vec<EntryFieldAliasDto>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct EntryFieldDto {
    pub id: Id,
    pub entry_id: Id,
    pub slug: Slug,
    pub override_value: String,
    pub dirty_value: String,
    pub clean_value: String,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<entry_fields::Model> for EntryFieldDto {
    type Error = DcAppError;

    fn try_from(value: entry_fields::Model) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id.into(),
            entry_id: value.entry_id.into(),
            slug: value.slug.try_into().map_err(DcAppError::database_src)?,
            override_value: value.override_value,
            dirty_value: value.dirty_value,
            clean_value: value.clean_value,
            updated_at: value.updated_at,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CreateEntryFieldDto {
    pub entry: EntryAliasDto,
    pub slug: Slug,
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
        am.slug = ActiveValue::Set(self.slug.into());
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
    pub entry_index: Index,
    pub vernacular_id: Id,
    pub vernacular_slug: Slug,
    pub owner_auth_user_id: Id,
    /// username may not be populated if not specifically called for
    pub owner_username: Option<Slug>,
}

impl EntryFieldWithContext {
    pub fn matches_ref(
        &self,
        alias: &EntryFieldRef,
        caller_id: &Id,
        entry_id: Option<&Id>,
    ) -> bool {
        let qualified_ef = match alias {
            EntryFieldRef::Id(id) => return *id == self.entry_field.id,
            EntryFieldRef::Qualified(alias) => alias,
        };
        if qualified_ef.slug != self.entry_field.slug {
            return false;
        }
        let qualified_entry = match &qualified_ef.entry {
            Some(EntryRef::Id(id)) => return *id == self.entry_field.entry_id,
            Some(EntryRef::Qualified(alias)) => alias,
            None => return entry_id == Some(&self.entry_field.entry_id),
        };
        if (qualified_entry.index.is_none() && entry_id == Some(&self.entry_field.entry_id))
            || qualified_entry.index.as_ref() != Some(&self.entry_index)
        {
            return false;
        }
        let ver_alias = match &qualified_entry.vernacular {
            VernacularRef::Id(id) => return *id == self.vernacular_id,
            VernacularRef::Qualified(alias) => alias,
        };
        if ver_alias.slug != self.vernacular_slug {
            return false;
        }
        match &ver_alias.user_alias {
            Some(UserAlias::Id(id)) => *id.id() == self.owner_auth_user_id,
            Some(UserAlias::Username(username)) => {
                Some(username.username()) == self.owner_username.as_ref()
            }
            None => *caller_id == self.owner_auth_user_id,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct FoundManyEntryFields {
    pub entry_fields: Vec<EntryFieldWithContext>,
    #[schema(value_type = HashMap<String, Option<usize>>)]
    pub aliases: HashMap<EntryFieldAlias, Option<usize>>,
}
