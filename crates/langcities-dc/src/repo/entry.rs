use langcities_common_server::dto::request::RequestContext;
use langcities_lcdcdsl::component::EntryAlias;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, EntityTrait, ExprTrait,
    IntoActiveModel, QueryFilter, TransactionError, TransactionTrait,
    sea_query::{Expr, Query},
};
use serde::{Deserialize, Serialize};

use crate::{
    dto::entries::CreateEntryDto,
    entity::{entries, vernaculars},
    error::{DcAppError, DcAppErrorTrait},
    repo::vernacular::{VernacularAction, VernacularRepo},
    state::AppState,
};

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

#[derive(Clone, Debug)]
pub struct EntryRepo {
    pub vernacular_repo: VernacularRepo,
}

impl EntryRepo {
    pub fn new(vernacular_repo: impl Into<VernacularRepo>) -> Self {
        let vernacular_repo = vernacular_repo.into();
        Self { vernacular_repo }
    }

    pub fn from_state(state: impl Into<AppState>) -> Self {
        Self::new(VernacularRepo::new(state))
    }

    pub fn state(&self) -> &AppState {
        &self.vernacular_repo.state
    }

    pub async fn generate_filter(
        &self,
        alias: EntryAlias,
        context: RequestContext,
        action: EntryAction,
    ) -> Result<Expr, DcAppError> {
        let (entry_expr, ver_expr) = match alias {
            EntryAlias::Id(id) => {
                let id_expr = entries::Column::Id.eq(*id);
                if action.enforce_owner() {
                    if let Some(auth_user_id) = context.caller_id {
                        (
                            id_expr,
                            Some(VernacularRepo::generate_owner_filter(*auth_user_id)),
                        )
                    } else {
                        return Err(DcAppError::unauthorized("unauthorized"));
                    }
                } else {
                    (id_expr, None)
                }
            }
            EntryAlias::Alias(alias) => {
                let index_expr = entries::Column::Index.eq(*alias.index);
                let ver_expr = self
                    .vernacular_repo
                    .generate_filter(
                        alias.vernacular_alias,
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

    pub(crate) async fn inner_get_entry<C>(
        &self,
        conn: &C,
        alias: EntryAlias,
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
            .map_err(DcAppError::database)
    }

    pub async fn get_entry<C>(
        &self,
        conn: &C,
        alias: EntryAlias,
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
                dto.vernacular.clone().0,
                context,
                VernacularAction::Write,
            )
            .await?
            .ok_or_else(|| {
                DcAppError::not_found(format!("vernacular {} not found", dto.vernacular))
            })?;
        let entry = entries::Entity::insert(dto.to_active_model(&vernacular))
            .exec_with_returning(conn)
            .await
            .map_err(DcAppError::database)?;
        let next_entry_index = vernacular.next_entry_id + 1;
        let mut v = vernacular.into_active_model();
        v.next_entry_id = ActiveValue::Set(next_entry_index);
        v.update(conn).await.map_err(DcAppError::database)?;
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
        .map_err(|e| match e {
            TransactionError::Connection(e) => DcAppError::database(e),
            TransactionError::Transaction(e) => e,
        })
    }

    pub async fn delete_entry<C>(
        &self,
        conn: &C,
        alias: EntryAlias,
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
            .map_err(DcAppError::database)?;
        Ok(Some(entry))
    }
}
