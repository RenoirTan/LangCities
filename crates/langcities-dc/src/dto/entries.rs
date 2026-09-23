use std::fmt::Display;

use chrono::{DateTime, Utc};
use langcities_common_server::dto::request::RequestContext;
use langcities_lcdcdsl::component::{EntryAlias, Id};
use sea_orm::{
    ActiveValue, ColumnTrait, ConnectionTrait, EntityTrait, ExprTrait, QueryFilter,
    sea_query::{Expr, Query},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    dto::vernaculars::VernacularAliasDto,
    entity::{entries, vernaculars},
    error::{DcAppError, DcAppErrorTrait},
    state::AppState,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[schema(value_type = String)]
pub struct EntryAliasDto(pub EntryAlias);

impl EntryAliasDto {
    pub(crate) async fn generate_filter(
        &self,
        caller_id: Option<Id>,
        state: &AppState,
        enforce_owner: bool,
    ) -> Result<Expr, DcAppError> {
        let cond = match &self.0 {
            EntryAlias::Id(id) => {
                let expr = entries::Column::Id.eq(**id);
                if enforce_owner {
                    let ver_expr = VernacularAliasDto::generate_owner_enforced(caller_id)?;
                    expr.and(
                        entries::Column::VernacularId.in_subquery(
                            Query::select()
                                .column(vernaculars::Column::Id)
                                .and_where(ver_expr)
                                .from(vernaculars::Entity)
                                .to_owned(),
                        ),
                    )
                } else {
                    expr
                }
            }
            EntryAlias::Alias(alias) => {
                let index_expr = entries::Column::Index.eq(*alias.index);
                let ver_expr = VernacularAliasDto(alias.vernacular_alias.clone())
                    .generate_filter(caller_id, state, enforce_owner)
                    .await?;
                index_expr.and(
                    entries::Column::VernacularId.in_subquery(
                        Query::select()
                            .column(vernaculars::Column::Id)
                            .and_where(ver_expr)
                            .from(vernaculars::Entity)
                            .to_owned(),
                    ),
                )
            }
        };
        Ok(cond)
    }
}

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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryAccessDto {
    pub alias: EntryAliasDto,
    pub request_context: RequestContext,
    pub action: EntryAction,
}

impl EntryAccessDto {
    pub fn read(alias: EntryAliasDto, request_context: RequestContext) -> Self {
        Self {
            alias,
            request_context,
            action: EntryAction::Read,
        }
    }

    pub fn write(alias: EntryAliasDto, request_context: RequestContext) -> Self {
        Self {
            alias,
            request_context,
            action: EntryAction::Write,
        }
    }

    pub fn delete(alias: EntryAliasDto, request_context: RequestContext) -> Self {
        Self {
            alias,
            request_context,
            action: EntryAction::Delete,
        }
    }

    pub async fn resolve<C: ConnectionTrait>(
        self,
        conn: &C,
        state: &AppState,
    ) -> Result<Option<entries::Model>, DcAppError> {
        let expr = self
            .alias
            .generate_filter(
                self.request_context.caller_id,
                state,
                self.action.enforce_owner(),
            )
            .await?;
        entries::Entity::find()
            .filter(expr)
            .one(conn)
            .await
            .map_err(DcAppError::database)
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
