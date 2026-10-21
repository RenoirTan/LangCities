use std::collections::{HashMap, HashSet};

use langcities_lcdcdsl::component::{EntryRef, Id, Slug, VernacularRef};
use langcities_server::dto::request::RequestContext;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, EntityTrait, ExprTrait,
    IntoActiveModel, QueryFilter, TransactionTrait,
    sea_query::{Expr, Query},
};
use serde::{Deserialize, Serialize};

use crate::{
    dto::entries::CreateEntryDto,
    entity::{entries, vernaculars},
    error::{DcAppError, DcAppErrorTrait},
    repo::vernacular::{VernacularAction, VernacularRepo},
    util::user::UserHelper,
};

#[derive(Clone)]
pub(super) struct FindManyEntryComponents {
    pub exprs: HashMap<EntryRef, Expr>,
    /// auth_user_id to username
    pub usernames: HashMap<Id, Slug>,
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

    pub fn to_vernacular_action(&self) -> VernacularAction {
        match self {
            Self::Read => VernacularAction::Read,
            Self::Write | Self::Delete => VernacularAction::Write,
        }
    }
}

#[derive(Clone)]
pub struct EntryRepo {
    vernacular_repo: VernacularRepo,
}

impl EntryRepo {
    pub fn new(vernacular_repo: impl Into<VernacularRepo>) -> Self {
        let vernacular_repo = vernacular_repo.into();
        Self { vernacular_repo }
    }

    #[inline]
    pub(super) fn user_helper(&self) -> &UserHelper {
        &self.vernacular_repo.user_helper()
    }

    #[inline]
    pub(super) fn vernacular_repo(&self) -> &VernacularRepo {
        &self.vernacular_repo
    }

    pub async fn generate_filter(
        &self,
        alias: EntryRef,
        context: RequestContext,
        action: EntryAction,
    ) -> Result<Expr, DcAppError> {
        let (entry_expr, ver_expr) = match alias {
            EntryRef::Id(id) => {
                let id_expr = entries::Column::Id.eq(*id);
                if action.enforce_owner() {
                    if let Some(auth_user_id) = context.caller_id {
                        (
                            id_expr,
                            Some(VernacularRepo::generate_owner_filter(*auth_user_id)),
                        )
                    } else {
                        return Err(DcAppError::unauthorized_desc("unauthorized"));
                    }
                } else {
                    (id_expr, None)
                }
            }
            EntryRef::Qualified(alias) => {
                let Some(index) = alias.index else {
                    return Err(DcAppError::bad_request_desc(
                        "entry cannot have wildcard index",
                    ));
                };
                let index_expr = entries::Column::Index.eq(*index);
                let ver_expr = self
                    .vernacular_repo
                    .generate_filter(
                        alias.vernacular.into(),
                        context,
                        action.to_vernacular_action(),
                    )
                    .await?;
                (index_expr, Some(ver_expr))
            }
        };
        let expr = if let Some(ver_expr) = ver_expr {
            entries::Column::VernacularId
                .in_subquery(
                    Query::select()
                        .from(vernaculars::Entity)
                        .column(vernaculars::Column::Id)
                        .and_where(ver_expr)
                        .to_owned(),
                )
                .and(entry_expr)
        } else {
            entry_expr
        };
        Ok(expr)
    }

    /// assumes join between entries, vernaculars and dc_users
    pub(super) async fn generate_many_filter<I>(
        &self,
        aliases: I,
        caller_id: Id,
        current_entry_id: Option<Id>,
    ) -> Result<FindManyEntryComponents, DcAppError>
    where
        I: IntoIterator<Item = EntryRef>,
    {
        let aliases = aliases.into_iter().collect::<Vec<_>>();
        let mut required_vernaculars = HashSet::<VernacularRef>::new();
        for alias in &aliases {
            let EntryRef::Qualified(alias) = alias else {
                continue;
            };
            required_vernaculars.insert(alias.vernacular.clone());
        }
        let ver_comps = self
            .vernacular_repo
            .generate_many_filter(required_vernaculars, caller_id)
            .await?;
        let mut exprs = HashMap::new();
        for alias in aliases {
            let expr = match &alias {
                EntryRef::Id(id) => entries::Column::Id.eq(**id),
                EntryRef::Qualified(alias) => {
                    let index_expr = entries::Column::Index.eq(**alias
                        .index
                        .as_ref()
                        .or_else(|| current_entry_id.as_ref())
                        .ok_or_else(|| {
                            DcAppError::bad_request_desc(
                                "cannot find entry with wildcard index without current entry",
                            )
                        })?);
                    // TODO: hashmap of VernacularRef instead of VernacularAlias
                    let ver_expr = ver_comps.exprs.get(&alias.vernacular).unwrap().clone();
                    index_expr.and(ver_expr)
                }
            };
            exprs.insert(alias, expr);
        }
        Ok(FindManyEntryComponents {
            exprs,
            usernames: ver_comps.usernames,
        })
    }

    pub(crate) async fn inner_get_entry<C>(
        &self,
        conn: &C,
        alias: EntryRef,
        context: RequestContext,
        action: EntryAction,
    ) -> Result<Option<entries::Model>, DcAppError>
    where
        C: ConnectionTrait,
    {
        let expr = self.generate_filter(alias, context, action).await?;
        entries::Entity::find()
            .filter(expr)
            .one(conn)
            .await
            .map_err(DcAppError::database_src)
    }

    pub async fn get_entry<C>(
        &self,
        conn: &C,
        alias: EntryRef,
        context: RequestContext,
    ) -> Result<Option<entries::Model>, DcAppError>
    where
        C: ConnectionTrait,
    {
        self.inner_get_entry(conn, alias, context, EntryAction::Read)
            .await
    }

    async fn inner_create_entry<C>(
        &self,
        conn: &C,
        dto: CreateEntryDto,
        context: RequestContext,
    ) -> Result<entries::Model, DcAppError>
    where
        C: ConnectionTrait,
    {
        let vernacular = self
            .vernacular_repo
            .inner_get_vernacular(
                conn,
                dto.vernacular.clone().0.into(),
                context,
                VernacularAction::Write,
            )
            .await?
            .ok_or_else(|| {
                DcAppError::not_found_desc(format!("vernacular {} not found", dto.vernacular))
            })?;
        let entry = entries::Entity::insert(dto.to_active_model(&vernacular))
            .exec_with_returning(conn)
            .await
            .map_err(DcAppError::database_src)?;
        let next_entry_index = vernacular.next_entry_id + 1;
        let mut v = vernacular.into_active_model();
        v.next_entry_id = ActiveValue::Set(next_entry_index);
        v.update(conn).await.map_err(DcAppError::database_src)?;
        Ok(entry)
    }

    pub async fn create_entry<C>(
        &self,
        conn: &C,
        dto: CreateEntryDto,
        context: RequestContext,
    ) -> Result<entries::Model, DcAppError>
    where
        C: TransactionTrait,
    {
        let me = self.clone();
        conn.transaction(|txn| {
            Box::pin(async move { me.inner_create_entry(txn, dto, context).await })
        })
        .await
        .map_err(DcAppError::flatten_tx)
    }

    pub async fn delete_entry<C>(
        &self,
        conn: &C,
        alias: EntryRef,
        context: RequestContext,
    ) -> Result<Option<entries::Model>, DcAppError>
    where
        C: ConnectionTrait,
    {
        let entry = match self
            .inner_get_entry(conn, alias, context, EntryAction::Delete)
            .await?
        {
            Some(model) => model,
            None => return Ok(None),
        };
        entries::Entity::delete_by_id(entry.id)
            .exec(conn)
            .await
            .map_err(DcAppError::database_src)?;
        Ok(Some(entry))
    }
}
