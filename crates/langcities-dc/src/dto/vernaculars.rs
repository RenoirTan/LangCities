use std::fmt::Display;

use chrono::Utc;
use langcities_lcdcdsl::component::{Id, Slug, VernacularAlias};
use sea_orm::{ActiveValue, entity::prelude::DateTimeUtc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    entity::vernaculars,
    error::{DcAppError, DcAppErrorTrait},
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[schema(value_type = String)]
pub struct VernacularAliasDto(pub VernacularAlias);

impl Display for VernacularAliasDto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct VernacularDto {
    pub id: Id,
    pub slug: Slug,
    pub name: String,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTimeUtc,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTimeUtc,
    pub owner_id: i64,
    pub next_entry_id: i64,
}

impl TryFrom<vernaculars::Model> for VernacularDto {
    type Error = DcAppError;

    fn try_from(model: vernaculars::Model) -> Result<Self, Self::Error> {
        Ok(Self {
            id: model.id.into(),
            slug: model.slug.parse().map_err(DcAppError::database_src)?,
            name: model.name,
            updated_at: model.updated_at,
            created_at: model.created_at,
            owner_id: model.owner_id,
            next_entry_id: model.next_entry_id,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CreateVernacularDto {
    pub slug: Slug,
    pub name: String,
}

impl CreateVernacularDto {
    pub fn to_active_model(self, owner_id: Id) -> vernaculars::ActiveModel {
        vernaculars::ActiveModel {
            slug: ActiveValue::Set(self.slug.into()),
            name: ActiveValue::Set(self.name),
            owner_id: ActiveValue::Set(owner_id.into()),
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct UpdateVernacularDto {
    pub slug: Option<Slug>,
    pub name: Option<String>,
}

impl UpdateVernacularDto {
    pub fn to_active_model(self) -> vernaculars::ActiveModel {
        let mut am = vernaculars::ActiveModel::default();
        self.update_active_model(&mut am);
        am
    }

    pub fn update_active_model(
        self,
        am: &mut vernaculars::ActiveModel,
    ) -> &mut vernaculars::ActiveModel {
        if let Some(slug) = self.slug {
            am.slug = ActiveValue::Set(slug.into());
        }
        if let Some(name) = self.name {
            am.name = ActiveValue::Set(name);
        }
        am.updated_at = ActiveValue::Set(Utc::now());
        am
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use sea_orm::sea_query::SqliteQueryBuilder;

    #[test]
    fn deserialize_numeric_vernacular_alias() {
        let alias: VernacularAliasDto = serde_json::from_str(r#""123""#).unwrap();

        assert!(matches!(alias.0, VernacularAlias::Id(id) if *id == 123));
    }

    #[test]
    fn deserialize_named_vernacular_alias() {
        let alias: VernacularAliasDto = serde_json::from_str(r#""lang@someone""#).unwrap();

        assert!(
            matches!(alias.0, VernacularAlias::Qualified(alias) if alias.to_string() == "lang@someone")
        );
    }

    /*
    #[test]
    fn numeric_owner_aliases_resolve_auth_user_ids() {
        let query = Query::select()
            .column(vernaculars::Column::Id)
            .from(vernaculars::Entity)
            .and_where(VernacularAliasDto::generate_owner_from_auth_user_id(42))
            .to_owned()
            .to_string(SqliteQueryBuilder);

        assert!(
            query.contains(
                r#""vernaculars"."owner_id" IN (SELECT "id" FROM "dc_users" WHERE "dc_users"."auth_user_id" = 42)"#
            ),
            "{query}"
        );
    }
    */
}
