use std::fmt::Display;

use langcities_lcdcdsl::component::AliasedResourceId;
use sea_orm::{ActiveValue, entity::prelude::DateTimeUtc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::entity::vernaculars;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[schema(value_type = String)]
pub struct VernacularAliasDto(pub AliasedResourceId);

impl<'de> Deserialize<'de> for VernacularAliasDto {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse::<AliasedResourceId>()
            .map(Self)
            .map_err(serde::de::Error::custom)
    }
}

impl Display for VernacularAliasDto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/*
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, IntoParams)]
pub struct VernacularsGetQueryDto {
    pub identifier: VernacularSlugOwnerIdDto,
}
*/

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VernacularsDto {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub updated_at: DateTimeUtc,
    pub created_at: DateTimeUtc,
    pub owner_id: i64,
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct VernacularsCreateDto {
    pub slug: String,
    pub name: String,
}

impl VernacularsCreateDto {
    pub fn to_active_model(self, owner_id: i64) -> vernaculars::ActiveModel {
        vernaculars::ActiveModel {
            slug: ActiveValue::Set(self.slug),
            name: ActiveValue::Set(self.name),
            owner_id: ActiveValue::Set(owner_id),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_numeric_vernacular_alias() {
        let alias: VernacularAliasDto = serde_json::from_str(r#""123""#).unwrap();

        assert!(matches!(alias.0, AliasedResourceId::Id(id) if *id == 123));
    }

    #[test]
    fn deserialize_named_vernacular_alias() {
        let alias: VernacularAliasDto = serde_json::from_str(r#""lang@someone""#).unwrap();

        assert!(
            matches!(alias.0, AliasedResourceId::Alias(alias) if alias.to_string() == "lang@someone")
        );
    }
}
