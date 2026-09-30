use chrono::Utc;
use langcities_lcdcdsl::component::Id;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, EntityTrait, ExprTrait,
    IntoActiveModel, ModelTrait, QueryFilter, QueryOrder, TransactionError, TransactionTrait,
    sea_query::Expr,
};

use crate::{
    dto::entry_field_jobs::{CreateEntryFieldJobDto, EntryFieldJobStatus, UpdateEntryFieldJobDto},
    entity::entry_field_jobs,
    error::{DcAppError, DcAppErrorTrait},
    repo::entry_field::EntryFieldRepo,
    state::AppState,
};

#[derive(Clone, Debug)]
pub struct EntryFieldJobRepo {
    pub entry_field_repo: EntryFieldRepo,
}

impl EntryFieldJobRepo {
    pub fn new(entry_field_repo: impl Into<EntryFieldRepo>) -> Self {
        let entry_field_repo = entry_field_repo.into();
        Self { entry_field_repo }
    }

    pub fn from_state(state: impl Into<AppState>) -> Self {
        Self::new(EntryFieldRepo::from_state(state))
    }

    #[inline]
    pub fn state(&self) -> &AppState {
        self.entry_field_repo.state()
    }

    pub async fn generate_active_filter_for(&self, entry_field_id: Id) -> Expr {
        let field_expr = entry_field_jobs::Column::EntryFieldId.eq(*entry_field_id);
        let status_expr = entry_field_jobs::Column::Status.is_in(EntryFieldJobStatus::active());
        let expiry_expr = entry_field_jobs::Column::ExpiresAt.lt(Utc::now());
        field_expr.and(status_expr).and(expiry_expr)
    }

    pub async fn get_entry_field_job<C>(
        &self,
        conn: &C,
        id: Id,
    ) -> Result<Option<entry_field_jobs::Model>, DcAppError>
    where
        C: ConnectionTrait,
    {
        entry_field_jobs::Entity::find_by_id(*id)
            .one(conn)
            .await
            .map_err(DcAppError::database)
    }

    pub async fn find_active_entry_field_jobs_for<C>(
        &self,
        conn: &C,
        entry_field_id: Id,
    ) -> Result<Vec<entry_field_jobs::Model>, DcAppError>
    where
        C: ConnectionTrait,
    {
        entry_field_jobs::Entity::find()
            .filter(self.generate_active_filter_for(entry_field_id).await)
            // last to expire first
            .order_by_desc(entry_field_jobs::Column::ExpiresAt)
            .all(conn)
            .await
            .map_err(DcAppError::database)
    }

    /*
    async fn refresh_entry_field_job<C>(
        &self,
        conn: &C,
        id: Id,
    ) -> Result<Option<entry_field_jobs::Model>, DcAppError>
    where
        C: ConnectionTrait,
    {
        let expires_at = Utc::now() + self.state().config.dc.entry_field_job_ttl;
        let am = entry_field_jobs::ActiveModel {
            id: ActiveValue::Unchanged(*id),
            expires_at: ActiveValue::Set(expires_at),
            ..Default::default()
        };
        am.update(conn)
            .await
            .map(Some)
            .map_err(DcAppError::database)
    }
    */

    async fn refresh_latest_active_entry_field_job_for<C>(
        &self,
        conn: &C,
        entry_field_id: Id,
    ) -> Result<Option<entry_field_jobs::Model>, DcAppError>
    where
        C: ConnectionTrait,
    {
        let Some(latest) = entry_field_jobs::Entity::find()
            .filter(self.generate_active_filter_for(entry_field_id).await)
            .order_by_desc(entry_field_jobs::Column::ExpiresAt)
            .one(conn)
            .await
            .map_err(DcAppError::database)?
        else {
            return Ok(None);
        };
        let mut am = latest.into_active_model();
        am.expires_at = ActiveValue::Set(Utc::now() + self.state().config.dc.entry_field_job_ttl);
        am.update(conn)
            .await
            .map(Some)
            .map_err(DcAppError::database)
    }

    async fn inner_create_entry_field_job<C>(
        &self,
        conn: &C,
        dto: CreateEntryFieldJobDto,
    ) -> Result<entry_field_jobs::Model, DcAppError>
    where
        C: ConnectionTrait,
    {
        let expires_at = Utc::now() + self.state().config.dc.entry_field_job_ttl;
        let am = dto.to_active_model(expires_at);
        am.insert(conn).await.map_err(DcAppError::database)
    }

    async fn inner_create_or_refresh_entry_field_job<C>(
        &self,
        conn: &C,
        dto: CreateEntryFieldJobDto,
    ) -> Result<entry_field_jobs::Model, DcAppError>
    where
        C: ConnectionTrait,
    {
        if let Some(model) = self
            .refresh_latest_active_entry_field_job_for(conn, dto.entry_field_id.into())
            .await?
        {
            Ok(model)
        } else {
            self.inner_create_entry_field_job(conn, dto).await
        }
    }

    pub async fn create_or_refresh_entry_field_job<C>(
        &self,
        conn: &C,
        dto: CreateEntryFieldJobDto,
    ) -> Result<entry_field_jobs::Model, DcAppError>
    where
        C: ConnectionTrait + TransactionTrait,
    {
        let me = self.clone();
        conn.transaction(|txn| {
            Box::pin(async move { me.inner_create_or_refresh_entry_field_job(txn, dto).await })
        })
        .await
        .map_err(|e| match e {
            TransactionError::Connection(e) => DcAppError::database(e),
            TransactionError::Transaction(e) => e,
        })
    }

    pub async fn update_entry_field_job<C>(
        &self,
        conn: &C,
        id: Id,
        dto: UpdateEntryFieldJobDto,
    ) -> Result<Option<entry_field_jobs::Model>, DcAppError>
    where
        C: ConnectionTrait,
    {
        let Some(model) = self.get_entry_field_job(conn, id).await? else {
            return Ok(None);
        };
        let mut am = model.into_active_model();
        dto.update_active_model(&mut am)?;
        am.update(conn)
            .await
            .map(Some)
            .map_err(DcAppError::database)
    }

    async fn inner_delete_entry_field_job<C>(
        &self,
        conn: &C,
        id: Id,
    ) -> Result<Option<entry_field_jobs::Model>, DcAppError>
    where
        C: ConnectionTrait,
    {
        let Some(model) = self.get_entry_field_job(conn, id.clone()).await? else {
            return Ok(None);
        };
        let status = model.status.parse::<EntryFieldJobStatus>()?;
        let now = Utc::now();
        if status.is_active() && now <= model.expires_at {
            return Err(DcAppError::bad_state(format!(
                "cannot delete active entry field job {id}"
            )));
        }
        model
            .clone()
            .delete(conn)
            .await
            .map_err(DcAppError::database)?;
        Ok(Some(model))
    }

    pub async fn delete_entry_field_job<C>(
        &self,
        conn: &C,
        id: Id,
    ) -> Result<Option<entry_field_jobs::Model>, DcAppError>
    where
        C: ConnectionTrait + TransactionTrait,
    {
        let me = self.clone();
        conn.transaction(|txn| {
            Box::pin(async move { me.inner_delete_entry_field_job(txn, id).await })
        })
        .await
        .map_err(|e| match e {
            TransactionError::Connection(e) => DcAppError::database(e),
            TransactionError::Transaction(e) => e,
        })
    }
}
