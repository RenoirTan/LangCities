use std::fmt::Display;

use langcities_lcdcdsl::component::{Alias, AliasedResourceId, Id, SlugOwnerId};
use sea_orm::{
    ActiveValue, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter,
    entity::prelude::DateTimeUtc, sea_query::Query,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    entity::{dc_users, vernaculars},
    error::{DcAppError, DcAppErrorTrait},
    state::AppState,
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
        state: &AppState,
        caller_id: Option<Id>,
        enforce_ownership: bool,
    ) -> Result<Option<vernaculars::Model>, DcAppError> {
        let vernacular = match &self.0 {
            AliasedResourceId::Id(id) => Self::resolve_id(**id, conn).await,
            AliasedResourceId::Alias(aliased) => {
                Self::resolve_aliased(aliased.clone(), conn, state).await
            }
            AliasedResourceId::Slug(slug) => {
                let owner_id = caller_id.as_ref().ok_or_else(|| {
                    DcAppError::bad_request(Some(format!("pure slug '{slug}' needs login").into()))
                })?;
                let aliased = SlugOwnerId {
                    slug: slug.clone(),
                    user_alias: Alias::Id(owner_id.clone()),
                };
                Self::resolve_aliased(aliased, conn, state).await
            }
        }?;
        match vernacular {
            Some(v) if enforce_ownership => {
                if let Some(caller_id) = caller_id
                    && caller_id == v.owner_id.into()
                {
                    Ok(Some(v))
                } else {
                    Err(DcAppError::unauthorized(Some(
                        format!("could not access '{self}'").into(),
                    )))
                }
            }
            otherwise => Ok(otherwise),
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
        state: &AppState,
    ) -> Result<Option<vernaculars::Model>, DcAppError> {
        let slug: String = aliased.slug.into();
        let user_condition = match aliased.user_alias {
            Alias::Id(id) => vernaculars::Column::OwnerId.eq(*id),
            Alias::Slug(username) => {
                let auth_user_id = state.resolve_auth_user_id(&**username).await?;
                vernaculars::Column::OwnerId.in_subquery(
                    Query::select()
                        .column(dc_users::Column::Id)
                        .and_where(dc_users::Column::AuthUserId.eq(auth_user_id))
                        .from(dc_users::Entity)
                        .to_owned(),
                )
            }
        };
        vernaculars::Entity::find()
            .filter(vernaculars::Column::Slug.eq(slug))
            .filter(user_condition)
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct VernacularDto {
    pub id: i64,
    pub slug: String,
    pub name: String,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTimeUtc,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTimeUtc,
    pub owner_id: i64,
}

impl From<vernaculars::Model> for VernacularDto {
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
pub struct CreateVernacularDto {
    pub slug: String,
    pub name: String,
}

impl CreateVernacularDto {
    pub fn to_active_model(self, owner_id: i64) -> vernaculars::ActiveModel {
        vernaculars::ActiveModel {
            slug: ActiveValue::Set(self.slug),
            name: ActiveValue::Set(self.name),
            owner_id: ActiveValue::Set(owner_id),
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct UpdateVernacularDto {
    pub slug: Option<String>,
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
            am.slug = ActiveValue::Set(slug);
        }
        if let Some(name) = self.name {
            am.name = ActiveValue::Set(name);
        }
        am
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
