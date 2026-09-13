use std::fmt::Display;

use langcities_lcdcdsl::component::{Alias, AliasedResourceId, SlugOwnerId};
use sea_orm::{
    ActiveValue, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter,
    entity::prelude::DateTimeUtc,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    entity::vernaculars,
    error::{DcAppError, DcAppErrorTrait},
};

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

impl VernacularAliasDto {
    pub async fn resolve<C: ConnectionTrait>(
        &self,
        conn: &C,
    ) -> Result<Option<vernaculars::Model>, DcAppError> {
        match &self.0 {
            AliasedResourceId::Id(id) => Self::resolve_id(**id, conn).await,
            AliasedResourceId::Alias(aliased) => Self::resolve_aliased(aliased.clone(), conn).await,
        }
    }

    async fn resolve_id<C: ConnectionTrait>(
        id: i64,
        conn: &C,
    ) -> Result<Option<vernaculars::Model>, DcAppError> {
        vernaculars::Entity::find_by_id(id)
            .one(conn)
            .await
            // .map(|o| o.map(|m| m.id))
            .map_err(|e| DcAppError::database(Some(e.into())))
    }

    async fn resolve_aliased<C: ConnectionTrait>(
        aliased: SlugOwnerId,
        conn: &C,
    ) -> Result<Option<vernaculars::Model>, DcAppError> {
        let slug: String = aliased.slug.into();
        let user_id = match aliased.user_alias {
            Alias::Id(id) => id,
            Alias::Slug(_) => {
                return Err(DcAppError::bad_request(Some(
                    "user slug not implemented".into(),
                )));
            }
        };
        vernaculars::Entity::find()
            .filter(vernaculars::Column::Slug.eq(slug))
            .filter(vernaculars::Column::OwnerId.eq(*user_id))
            .one(conn)
            .await
            // .map(|o| o.map(|m| m.id))
            .map_err(|e| DcAppError::database(Some(e.into())))
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
