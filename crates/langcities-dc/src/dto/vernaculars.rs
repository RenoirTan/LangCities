use std::fmt::Display;

use chrono::Utc;
use langcities_common_server::dto::request::{RequestAccessKind, RequestContext};
use langcities_lcdcdsl::component::{Alias, Id, VernacularAlias};
use sea_orm::{
    ActiveValue, ColumnTrait, ConnectionTrait, EntityTrait, ExprTrait, QueryFilter,
    entity::prelude::DateTimeUtc,
    sea_query::{Expr, Query},
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
pub struct VernacularAliasDto(pub VernacularAlias);

impl VernacularAliasDto {
    pub(crate) async fn generate_filter(
        &self,
        caller_id: Option<Id>,
        state: &AppState,
        enforce_owner: bool,
    ) -> Result<Expr, DcAppError> {
        let cond = match &self.0 {
            VernacularAlias::Id(id) => vernaculars::Column::Id.eq(**id),
            VernacularAlias::Alias(alias) => {
                let slug_expr = vernaculars::Column::Slug.eq(&**alias.slug);
                let owner_expr = match &alias.user_alias {
                    Alias::Id(id) => vernaculars::Column::OwnerId.eq(**id),
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
                slug_expr.and(owner_expr)
            }
            VernacularAlias::Slug(slug) => {
                if let Some(caller_id) = &caller_id {
                    vernaculars::Column::Slug
                        .eq(&**slug)
                        .and(vernaculars::Column::OwnerId.eq(**caller_id))
                } else {
                    return Err(DcAppError::unauthorized(Some("".into())));
                }
            }
        };
        if enforce_owner {
            Ok(cond.and(Self::generate_owner_enforced(caller_id)?))
        } else {
            Ok(cond)
        }
    }

    pub(crate) fn generate_owner_enforcement(caller_id: Id) -> Expr {
        vernaculars::Column::OwnerId.eq(*caller_id)
    }

    pub(crate) fn generate_owner_enforced(caller_id: Option<Id>) -> Result<Expr, DcAppError> {
        caller_id
            .map(|id| Self::generate_owner_enforcement(id))
            .ok_or_else(|| DcAppError::unauthorized(None))
    }
}

impl<'de> Deserialize<'de> for VernacularAliasDto {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse::<VernacularAlias>()
            .map(Self)
            .map_err(serde::de::Error::custom)
    }
}

impl Display for VernacularAliasDto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum VernacularAction {
    Read,
    Write,
    Delete,
}

impl VernacularAction {
    pub fn enforce_owner(&self) -> bool {
        !matches!(self, Self::Read)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VernacularAccessDto {
    pub alias: VernacularAliasDto,
    pub request_context: RequestContext,
    pub action: VernacularAction,
}

impl VernacularAccessDto {
    pub fn read(alias: VernacularAliasDto, request_context: RequestContext) -> Self {
        Self {
            alias,
            request_context,
            action: VernacularAction::Read,
        }
    }

    pub fn write(alias: VernacularAliasDto, request_context: RequestContext) -> Self {
        Self {
            alias,
            request_context,
            action: VernacularAction::Write,
        }
    }

    pub fn delete(alias: VernacularAliasDto, request_context: RequestContext) -> Self {
        Self {
            alias,
            request_context,
            action: VernacularAction::Delete,
        }
    }

    pub fn can_access(&self, vernacular: &vernaculars::Model) -> bool {
        match &self.action {
            VernacularAction::Read => true,
            VernacularAction::Write | VernacularAction::Delete => {
                match &self.request_context.access_kind {
                    RequestAccessKind::NormalUser => {
                        if let Some(caller_id) = &self.request_context.caller_id {
                            Id::from(vernacular.owner_id) == *caller_id
                        } else {
                            false
                        }
                    }
                }
            }
        }
    }

    pub async fn resolve<C: ConnectionTrait>(
        self,
        conn: &C,
        state: &AppState,
    ) -> Result<Option<vernaculars::Model>, DcAppError> {
        let caller_id = self.request_context.caller_id;
        let enforce_owner = self.action.enforce_owner();
        let expr = self
            .alias
            .generate_filter(caller_id, state, enforce_owner)
            .await?;
        vernaculars::Entity::find()
            .filter(expr)
            .one(conn)
            .await
            .map_err(|e| DcAppError::database(Some(e.into())))
    }
}

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
    pub next_entry_id: i64,
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
            next_entry_id: model.next_entry_id,
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
        am.updated_at = ActiveValue::Set(Utc::now());
        am
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_numeric_vernacular_alias() {
        let alias: VernacularAliasDto = serde_json::from_str(r#""123""#).unwrap();

        assert!(matches!(alias.0, VernacularAlias::Id(id) if *id == 123));
    }

    #[test]
    fn deserialize_named_vernacular_alias() {
        let alias: VernacularAliasDto = serde_json::from_str(r#""lang@someone""#).unwrap();

        assert!(
            matches!(alias.0, VernacularAlias::Alias(alias) if alias.to_string() == "lang@someone")
        );
    }
}
