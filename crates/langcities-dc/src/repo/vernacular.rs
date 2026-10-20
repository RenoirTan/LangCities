use std::collections::{HashMap, HashSet};

use langcities_lcdcdsl::component::{Id, QualifiedVernacular, Slug, UserAlias, VernacularRef};
use langcities_server::dto::request::RequestContext;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, EntityTrait, ExprTrait,
    IntoActiveModel, QueryFilter, QuerySelect, TransactionTrait, TryInsertResult, sea_query::Expr,
};
use serde::{Deserialize, Serialize};

use crate::{
    dto::vernaculars::{CreateVernacularDto, UpdateVernacularDto},
    entity::{dc_users, entries, vernaculars},
    error::{DcAppError, DcAppErrorTrait},
    state::AppState,
};

/// Only for finding entry field dependencies
#[derive(Clone)]
pub(super) struct FindManyVernacularComponents {
    pub exprs: HashMap<VernacularRef, Expr>,
    /// auth_user_id to username
    pub usernames: HashMap<Id, Slug>,
}

/// Simple permissions enum
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum VernacularAction {
    Read,
    Write,
    Delete,
}

impl VernacularAction {
    pub const fn enforce_owner(&self) -> bool {
        !matches!(self, Self::Read)
    }
}

#[derive(Clone)]
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
        alias: VernacularRef,
        context: RequestContext,
        action: VernacularAction,
    ) -> Result<Expr, DcAppError> {
        let expr = match alias {
            VernacularRef::Id(id) => vernaculars::Column::Id.eq(*id),
            VernacularRef::Qualified(alias) => {
                let slug_expr = vernaculars::Column::Slug.eq(&**alias.slug);
                let owner_expr = match &alias.user_alias {
                    Some(UserAlias::Id(owner_id)) => Self::generate_owner_filter(**owner_id.id()),
                    Some(UserAlias::Username(username)) => {
                        let auth_user_id = self
                            .state
                            .resolve_auth_user_id(&username.username())
                            .await?;
                        Self::generate_owner_filter(*auth_user_id)
                    }
                    None => match &context.caller_id {
                        Some(id) => Self::generate_owner_filter(**id),
                        None => {
                            return Err(DcAppError::bad_request_desc(
                                "no owner id in alias or request context",
                            ));
                        }
                    },
                };
                slug_expr.and(owner_expr)
            }
        };
        if action.enforce_owner() {
            if let Some(caller_id) = &context.caller_id {
                Ok(Self::generate_owner_filter(**caller_id).and(expr))
            } else {
                Err(DcAppError::unauthorized_desc("unauthorized"))
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

    /// assumes that dc_users has been joined with vernaculars
    #[inline]
    fn generate_joined_owner_filter(auth_user_id: i64) -> Expr {
        dc_users::Column::AuthUserId.eq(auth_user_id)
    }

    /// only for use with finding dependencies of entry fields
    ///
    /// assumes that dc_users has been joined with vernaculars
    pub(super) async fn generate_many_filter<I>(
        &self,
        aliases: I,
        caller_id: Id,
    ) -> Result<FindManyVernacularComponents, DcAppError>
    where
        I: IntoIterator<Item = VernacularRef>,
    {
        let aliases = aliases.into_iter().collect::<Vec<_>>();
        let mut required_usernames = HashSet::<Slug>::new();
        for alias in &aliases {
            let VernacularRef::Qualified(alias) = alias else {
                continue;
            };
            let Some(UserAlias::Username(username)) = &alias.user_alias else {
                continue;
            };
            required_usernames.insert(username.username().clone().into());
        }
        let required_usernames = required_usernames.iter().collect::<Vec<_>>();
        let username_to_auth_user_ids = self
            .state
            .resolve_auth_user_ids(&required_usernames)
            .await?;
        let mut exprs = HashMap::new();
        for alias in aliases {
            let expr = match &alias {
                VernacularRef::Id(id) => vernaculars::Column::Id.eq(**id),
                VernacularRef::Qualified(alias) => {
                    let slug_expr = vernaculars::Column::Slug.eq(&**alias.slug);
                    let owner_auth_user_id = match &alias.user_alias {
                        Some(UserAlias::Id(id)) => &*id.id(),
                        Some(UserAlias::Username(username)) => &*username_to_auth_user_ids
                            .get(&*username.username())
                            .unwrap(),
                        None => &caller_id,
                    };
                    let owner_expr = Self::generate_joined_owner_filter(**owner_auth_user_id);
                    slug_expr.and(owner_expr)
                }
            };
            exprs.insert(alias, expr);
        }
        let usernames: HashMap<_, _> = username_to_auth_user_ids
            .into_iter()
            .map(|(k, v)| (v, k))
            .collect();
        let components = FindManyVernacularComponents { exprs, usernames };
        Ok(components)
    }

    pub(crate) async fn inner_get_vernacular<C>(
        &self,
        conn: &C,
        alias: VernacularRef,
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
            .map_err(DcAppError::database_src)
    }

    pub async fn get_vernacular<C>(
        &self,
        conn: &C,
        alias: VernacularRef,
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
        owner_auth_user_id: Id,
        context: RequestContext,
    ) -> Result<vernaculars::Model, DcAppError>
    where
        C: TransactionTrait,
    {
        let me = self.clone();
        conn.transaction(|txn| {
            Box::pin(async move {
                me.inner_create_vernacular(txn, dto, owner_auth_user_id, context)
                    .await
            })
        })
        .await
        .map_err(DcAppError::flatten_tx)
    }

    async fn inner_create_vernacular<C>(
        &self,
        conn: &C,
        dto: CreateVernacularDto,
        owner_auth_user_id: Id,
        context: RequestContext,
    ) -> Result<vernaculars::Model, DcAppError>
    where
        C: ConnectionTrait,
    {
        let slug = dto.slug.clone();
        let am = dto.to_active_model(owner_auth_user_id.clone());
        let result = vernaculars::Entity::insert(am)
            .on_conflict_do_nothing_on([vernaculars::Column::Slug, vernaculars::Column::OwnerId])
            .exec_with_returning(conn)
            .await
            .map_err(DcAppError::database_src)?;
        let vernacular = match result {
            TryInsertResult::Empty => {
                return Err(DcAppError::database_desc(
                    "no sql executed for create_vernacular",
                ));
            }
            TryInsertResult::Conflicted => self
                .get_vernacular(
                    conn,
                    VernacularRef::Qualified(QualifiedVernacular {
                        slug,
                        user_alias: Some(UserAlias::from_id(owner_auth_user_id)),
                    }),
                    context,
                )
                .await
                .map(|o| {
                    o.ok_or_else(|| DcAppError::database_desc("could not insert or get vernacular"))
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
        .map_err(DcAppError::database_src)?;

        Ok(vernacular)
    }

    pub async fn update_vernacular<C>(
        &self,
        conn: &C,
        alias: VernacularRef,
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
            .map_err(DcAppError::database_src)
    }

    pub async fn delete_vernacular<C>(
        &self,
        conn: &C,
        alias: VernacularRef,
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
                .map_err(DcAppError::database_src)?;
            Ok(Some(model))
        } else {
            Ok(None)
        }
    }
}
