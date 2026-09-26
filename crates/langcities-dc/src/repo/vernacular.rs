use langcities_common_server::dto::request::RequestContext;
use langcities_lcdcdsl::component::{AliasedVernacular, UserAlias, VernacularAlias};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, EntityTrait, ExprTrait,
    IntoActiveModel, QueryFilter, QuerySelect, TransactionError, TransactionTrait, TryInsertResult,
    sea_query::Expr,
};
use serde::{Deserialize, Serialize};

use crate::{
    dto::vernaculars::{CreateVernacularDto, UpdateVernacularDto},
    entity::{dc_users, entries, vernaculars},
    error::{DcAppError, DcAppErrorTrait},
    state::AppState,
};

/// Simple permissions enum
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

#[derive(Clone, Debug)]
pub struct VernacularRepo {
    pub state: AppState,
}

impl VernacularRepo {
    pub fn new(state: impl Into<AppState>) -> Self {
        let state = state.into();
        Self { state }
    }

    pub async fn generate_filter(
        &self,
        alias: VernacularAlias,
        context: RequestContext,
        action: VernacularAction,
    ) -> Result<Expr, DcAppError> {
        let expr = match alias {
            VernacularAlias::Id(id) => vernaculars::Column::Id.eq(*id),
            VernacularAlias::Alias(alias) => {
                let slug_expr = vernaculars::Column::Slug.eq(&**alias.slug);
                let owner_expr = match alias.user_alias {
                    UserAlias::Id(owner_id) => vernaculars::Column::OwnerId.eq(*owner_id),
                    UserAlias::Slug(username) => {
                        let auth_user_id = self.state.resolve_auth_user_id(&username).await?;
                        Self::generate_owner_filter(auth_user_id)
                    }
                };
                slug_expr.and(owner_expr)
            }
            VernacularAlias::Slug(slug) => {
                if let Some(caller_id) = &context.caller_id {
                    vernaculars::Column::Slug
                        .eq(&*slug)
                        .and(vernaculars::Column::OwnerId.eq(**caller_id))
                } else {
                    return Err(DcAppError::unauthorized(
                        "must be logged in to use slug format",
                    ));
                }
            }
        };
        if action.enforce_owner() {
            if let Some(caller_id) = &context.caller_id {
                Ok(Self::generate_owner_filter(**caller_id).and(expr))
            } else {
                Err(DcAppError::unauthorized("unauthorized"))
            }
        } else {
            Ok(expr)
        }
    }

    pub fn generate_owner_filter(auth_user_id: i64) -> Expr {
        let mut owner_expr = dc_users::Entity::find_by_auth_user_id(auth_user_id)
            .select_only()
            .column(dc_users::Column::Id);
        vernaculars::Column::OwnerId.in_subquery(QuerySelect::query(&mut owner_expr).to_owned())
    }

    pub(crate) async fn inner_get_vernacular<C>(
        &self,
        conn: &C,
        alias: VernacularAlias,
        context: RequestContext,
        action: VernacularAction,
    ) -> Result<Option<vernaculars::Model>, DcAppError>
    where
        C: ConnectionTrait,
    {
        let expr = self.generate_filter(alias, context, action).await?;
        vernaculars::Entity::find()
            .filter(expr)
            .one(conn)
            .await
            .map_err(DcAppError::database)
    }

    pub async fn get_vernacular<C>(
        &self,
        conn: &C,
        alias: VernacularAlias,
        context: RequestContext,
    ) -> Result<Option<vernaculars::Model>, DcAppError>
    where
        C: ConnectionTrait,
    {
        self.inner_get_vernacular(conn, alias, context, VernacularAction::Read)
            .await
    }

    pub async fn create_vernacular<C>(
        &self,
        conn: &C,
        dto: CreateVernacularDto,
        owner_auth_user_id: i64,
        context: RequestContext,
    ) -> Result<vernaculars::Model, DcAppError>
    where
        C: TransactionTrait,
    {
        let me = self.clone();
        match conn
            .transaction(|txn| {
                Box::pin(async move {
                    me.inner_create_vernacular(txn, dto, owner_auth_user_id, context)
                        .await
                })
            })
            .await
        {
            Ok(model) => Ok(model),
            Err(TransactionError::Transaction(e)) => Err(e),
            Err(TransactionError::Connection(e)) => Err(DcAppError::database(e)),
        }
    }

    async fn inner_create_vernacular<C>(
        &self,
        conn: &C,
        dto: CreateVernacularDto,
        owner_auth_user_id: i64,
        context: RequestContext,
    ) -> Result<vernaculars::Model, DcAppError>
    where
        C: ConnectionTrait,
    {
        let slug = dto.slug.clone();
        let am = dto.to_active_model(owner_auth_user_id);
        let result = vernaculars::Entity::insert(am)
            .on_conflict_do_nothing_on([vernaculars::Column::Slug, vernaculars::Column::OwnerId])
            .exec_with_returning(conn)
            .await
            .map_err(DcAppError::database)?;
        let vernacular = match result {
            TryInsertResult::Empty => {
                return Err(DcAppError::database(
                    "no sql executed for create_vernacular",
                ));
            }
            TryInsertResult::Conflicted => self
                .get_vernacular(
                    conn,
                    VernacularAlias::Alias(AliasedVernacular {
                        slug,
                        user_alias: UserAlias::Id(owner_auth_user_id.into()),
                    }),
                    context,
                )
                .await
                .map(|o| {
                    o.ok_or_else(|| DcAppError::database("could not insert or get vernacular"))
                })
                .flatten()?,
            TryInsertResult::Inserted(model) => model,
        };

        // TODO: move default entry to entries repo
        let _entry = entries::Entity::insert(entries::ActiveModel {
            index: ActiveValue::Set(0),
            vernacular_id: ActiveValue::Set(vernacular.id),
            ..Default::default()
        })
        .exec_with_returning(conn)
        .await
        .map_err(DcAppError::database)?;

        Ok(vernacular)
    }

    pub async fn update_vernacular<C>(
        &self,
        conn: &C,
        alias: VernacularAlias,
        dto: UpdateVernacularDto,
        context: RequestContext,
    ) -> Result<Option<vernaculars::Model>, DcAppError>
    where
        C: ConnectionTrait,
    {
        let mut vernacular = match self
            .inner_get_vernacular(conn, alias, context, VernacularAction::Write)
            .await?
        {
            Some(model) => model.into_active_model(),
            None => return Ok(None),
        };
        dto.update_active_model(&mut vernacular);
        vernacular
            .update(conn)
            .await
            .map(Some)
            .map_err(DcAppError::database)
    }

    pub async fn delete_vernacular<C>(
        &self,
        conn: &C,
        alias: VernacularAlias,
        context: RequestContext,
    ) -> Result<Option<vernaculars::Model>, DcAppError>
    where
        C: ConnectionTrait,
    {
        let model = self
            .inner_get_vernacular(conn, alias, context, VernacularAction::Delete)
            .await?;
        if let Some(model) = model {
            vernaculars::Entity::delete_by_id(model.id)
                .exec(conn)
                .await
                .map_err(DcAppError::database)?;
            Ok(Some(model))
        } else {
            Ok(None)
        }
    }
}
