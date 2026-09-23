use std::fmt::Display;

use chrono::{DateTime, Utc};
use langcities_common_server::dto::request::RequestContext;
use langcities_lcdcdsl::component::{EntryFieldAlias, Id};
use sea_orm::{
    ActiveValue, ColumnTrait, ConnectionTrait, EntityTrait, ExprTrait, QueryFilter, QuerySelect,
    sea_query::Expr,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    dto::{entries::EntryAliasDto, vernaculars::VernacularAliasDto},
    entity::{entries, entry_fields, vernaculars},
    error::{DcAppError, DcAppErrorTrait},
    state::AppState,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[schema(value_type = String)]
pub struct EntryFieldAliasDto(pub EntryFieldAlias);

impl Display for EntryFieldAliasDto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl EntryFieldAliasDto {
    pub(crate) async fn generate_filter(
        &self,
        caller_id: Option<Id>,
        state: &AppState,
        enforce_owner: bool,
    ) -> Result<Expr, DcAppError> {
        let cond = match &self.0 {
            EntryFieldAlias::Id(id) => {
                let expr = entry_fields::Column::Id.eq(**id);
                if enforce_owner {
                    let ver_expr = VernacularAliasDto::generate_owner_enforced(caller_id)?;
                    let mut entry_subquery = entries::Entity::find()
                        .left_join(vernaculars::Entity)
                        .filter(ver_expr)
                        .select_only()
                        .column(entries::Column::Id);
                    let entry_expr = entry_fields::Column::EntryId
                        .in_subquery(QuerySelect::query(&mut entry_subquery).to_owned());
                    expr.and(entry_expr)
                } else {
                    expr
                }
            }
            EntryFieldAlias::Alias(alias) => {
                let slug_expr = entry_fields::Column::Slug.eq(&**alias.slug);
                let entry_expr = EntryAliasDto(alias.entry_alias.clone())
                    .generate_filter(caller_id, state, enforce_owner)
                    .await?;
                slug_expr.and(
                    entry_fields::Column::EntryId.in_subquery(
                        QuerySelect::query(
                            &mut entries::Entity::find()
                                .filter(entry_expr)
                                .select_only()
                                .column(entries::Column::Id),
                        )
                        .to_owned(),
                    ),
                )
            }
        };
        Ok(cond)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum EntryFieldAction {
    Read,
    Write,
    Delete,
}

impl EntryFieldAction {
    pub fn enforce_owner(&self) -> bool {
        !matches!(self, Self::Read)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryFieldAccessDto {
    pub alias: EntryFieldAliasDto,
    pub request_context: RequestContext,
    pub action: EntryFieldAction,
}

impl EntryFieldAccessDto {
    pub fn read(alias: EntryFieldAliasDto, request_context: RequestContext) -> Self {
        Self {
            alias,
            request_context,
            action: EntryFieldAction::Read,
        }
    }

    pub fn write(alias: EntryFieldAliasDto, request_context: RequestContext) -> Self {
        Self {
            alias,
            request_context,
            action: EntryFieldAction::Write,
        }
    }

    pub fn delete(alias: EntryFieldAliasDto, request_context: RequestContext) -> Self {
        Self {
            alias,
            request_context,
            action: EntryFieldAction::Delete,
        }
    }

    pub async fn resolve<C: ConnectionTrait>(
        self,
        conn: &C,
        state: &AppState,
    ) -> Result<Option<entry_fields::Model>, DcAppError> {
        let expr = self
            .alias
            .generate_filter(
                self.request_context.caller_id,
                state,
                self.action.enforce_owner(),
            )
            .await?;
        entry_fields::Entity::find()
            .filter(expr)
            .one(conn)
            .await
            .map_err(DcAppError::database)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct EntryFieldDto {
    pub id: i64,
    pub entry_id: i64,
    pub slug: String,
    pub r#override: String,
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
            r#override: value.r#override,
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
    pub r#override: Option<String>,
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
        am.r#override = ActiveValue::Set(self.r#override.unwrap_or_default());
        am
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct UpdateEntryFieldDto {
    pub r#override: Option<String>,
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
        if let Some(o) = self.r#override {
            am.r#override = ActiveValue::Set(o);
        }
        am.updated_at = ActiveValue::Set(Utc::now());
        am
    }
}
