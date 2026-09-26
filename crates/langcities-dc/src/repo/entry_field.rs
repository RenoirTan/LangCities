use langcities_common_server::dto::request::RequestContext;
use langcities_lcdcdsl::component::EntryFieldAlias;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, ExprTrait, IntoActiveModel,
    QueryFilter, QuerySelect, TransactionError, TransactionTrait,
    sea_query::{Expr, Query},
};
use serde::{Deserialize, Serialize};

use crate::{
    dto::entry_fields::{CreateEntryFieldDto, UpdateEntryFieldDto},
    entity::{entries, entry_fields, vernaculars},
    error::{DcAppError, DcAppErrorTrait},
    repo::{
        entry::{EntryAction, EntryRepo},
        vernacular::{VernacularAction, VernacularRepo},
    },
    state::AppState,
};

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

    pub fn to_entry_action(&self) -> EntryAction {
        match self {
            Self::Read => EntryAction::Read,
            Self::Write | Self::Delete => EntryAction::Write,
        }
    }

    pub fn to_vernacular_action(&self) -> VernacularAction {
        match self {
            Self::Read => VernacularAction::Read,
            Self::Write | Self::Delete => VernacularAction::Write,
        }
    }
}

#[derive(Clone, Debug)]
pub struct EntryFieldRepo {
    pub entry_repo: EntryRepo,
}

impl EntryFieldRepo {
    pub fn new(entry_repo: impl Into<EntryRepo>) -> Self {
        let entry_repo = entry_repo.into();
        Self { entry_repo }
    }

    pub fn from_state(state: impl Into<AppState>) -> Self {
        Self::new(EntryRepo::from_state(state))
    }

    pub fn state(&self) -> &AppState {
        self.entry_repo.state()
    }

    pub fn vernacular_repo(&self) -> &VernacularRepo {
        &self.entry_repo.vernacular_repo
    }

    pub async fn generate_filter(
        &self,
        alias: EntryFieldAlias,
        context: RequestContext,
        action: EntryFieldAction,
    ) -> Result<Expr, DcAppError> {
        let (field_expr, entry_expr) = match &alias {
            EntryFieldAlias::Id(id) => {
                let id_expr = entry_fields::Column::Id.eq(**id);
                if action.enforce_owner() {
                    if let Some(auth_user_id) = &context.caller_id {
                        let ver_expr = VernacularRepo::generate_owner_filter(**auth_user_id);
                        let mut entry_subquery = entries::Entity::find()
                            .left_join(vernaculars::Entity)
                            .filter(ver_expr)
                            .select_only()
                            .column(entries::Column::Id);
                        let entry_expr = entry_fields::Column::EntryId
                            .in_subquery(QuerySelect::query(&mut entry_subquery).to_owned());
                        (id_expr, Some(entry_expr))
                    } else {
                        return Err(DcAppError::unauthorized("unauthorized"));
                    }
                } else {
                    (id_expr, None)
                }
            }
            EntryFieldAlias::Alias(alias) => {
                let slug_expr = entry_fields::Column::Slug.eq(&*alias.slug);
                let entry_expr = self
                    .entry_repo
                    .generate_filter(alias.entry_alias.clone(), context, action.to_entry_action())
                    .await?;
                (slug_expr, Some(entry_expr))
            }
        };
        let expr = if let Some(entry_expr) = entry_expr {
            entry_fields::Column::EntryId
                .in_subquery(
                    Query::select()
                        .from(entries::Entity)
                        .column(entries::Column::Id)
                        .and_where(entry_expr)
                        .to_owned(),
                )
                .and(field_expr)
        } else {
            field_expr
        };
        Ok(expr)
    }

    async fn inner_get_entry_field<C>(
        &self,
        conn: &C,
        alias: EntryFieldAlias,
        context: RequestContext,
        action: EntryFieldAction,
    ) -> Result<Option<entry_fields::Model>, DcAppError>
    where
        C: ConnectionTrait,
    {
        let expr = self.generate_filter(alias, context, action).await?;
        entry_fields::Entity::find()
            .filter(expr)
            .one(conn)
            .await
            .map_err(DcAppError::database)
    }

    pub async fn get_entry_field<C>(
        &self,
        conn: &C,
        alias: EntryFieldAlias,
        context: RequestContext,
    ) -> Result<Option<entry_fields::Model>, DcAppError>
    where
        C: ConnectionTrait,
    {
        self.inner_get_entry_field(conn, alias, context, EntryFieldAction::Read)
            .await
    }

    async fn inner_create_entry_field<C>(
        &self,
        conn: &C,
        dto: CreateEntryFieldDto,
        context: RequestContext,
    ) -> Result<entry_fields::Model, DcAppError>
    where
        C: ConnectionTrait,
    {
        let entry = self
            .entry_repo
            .inner_get_entry(conn, dto.entry.0.clone(), context, EntryAction::Write)
            .await?
            .ok_or_else(|| DcAppError::not_found(format!("entry {} not found", dto.entry)))?;
        let entry_field = entry_fields::Entity::insert(dto.to_active_model(&entry))
            .exec_with_returning(conn)
            .await
            .map_err(DcAppError::database)?;
        // TODO: add job
        Ok(entry_field)
    }

    pub async fn create_entry_field<C>(
        &self,
        conn: &C,
        dto: CreateEntryFieldDto,
        context: RequestContext,
    ) -> Result<entry_fields::Model, DcAppError>
    where
        C: TransactionTrait,
    {
        let me = self.clone();
        conn.transaction(|txn| {
            Box::pin(async move { me.inner_create_entry_field(txn, dto, context).await })
        })
        .await
        .map_err(|e| match e {
            TransactionError::Connection(e) => DcAppError::database(e),
            TransactionError::Transaction(e) => e,
        })
    }

    pub async fn update_entry_field<C>(
        &self,
        conn: &C,
        alias: EntryFieldAlias,
        dto: UpdateEntryFieldDto,
        context: RequestContext,
    ) -> Result<Option<entry_fields::Model>, DcAppError>
    where
        C: ConnectionTrait,
    {
        let mut entry_field = match self
            .inner_get_entry_field(conn, alias, context, EntryFieldAction::Write)
            .await?
        {
            Some(model) => model.into_active_model(),
            None => return Ok(None),
        };
        dto.update_active_model(&mut entry_field);
        entry_field
            .update(conn)
            .await
            .map(Some)
            .map_err(DcAppError::database)
        // TODO: trigger jobs
    }

    pub async fn delete_entry_field<C>(
        &self,
        conn: &C,
        alias: EntryFieldAlias,
        context: RequestContext,
    ) -> Result<Option<entry_fields::Model>, DcAppError>
    where
        C: ConnectionTrait,
    {
        let entry_field = match self
            .inner_get_entry_field(conn, alias, context, EntryFieldAction::Delete)
            .await?
        {
            Some(model) => model,
            None => return Ok(None),
        };
        entry_fields::Entity::delete_by_id(entry_field.id)
            .exec(conn)
            .await
            .map_err(DcAppError::database)?;
        Ok(Some(entry_field))
        // TODO: trigger jobs
    }
}
