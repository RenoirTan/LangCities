use chrono::{DateTime, Duration, TimeZone, Utc};
use langcities_lcdcdsl::component::Id;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, ExprTrait, IntoActiveModel,
    PaginatorTrait, QueryFilter, SelectExt, TransactionTrait, sea_query::Expr,
};

use crate::{
    dto::entry_field_job_attempts::{CreateEntryFieldJobAttemptDto, FinishEntryFieldJobAttemptDto},
    entity::entry_field_job_attempts,
    error::{DcAppError, DcAppErrorTrait},
    repo::{
        entry::EntryRepo, entry_field::EntryFieldRepo, entry_field_job::EntryFieldJobRepo,
        vernacular::VernacularRepo,
    },
    util::user::UserHelper,
};

#[derive(Clone)]
pub struct EntryFieldJobAttemptRepo {
    pub entry_field_job_repo: EntryFieldJobRepo,
}

impl EntryFieldJobAttemptRepo {
    pub fn new(entry_field_job_repo: impl Into<EntryFieldJobRepo>) -> Self {
        let entry_field_job_repo = entry_field_job_repo.into();
        Self {
            entry_field_job_repo,
        }
    }

    #[inline]
    pub(super) fn user_helper(&self) -> &UserHelper {
        self.entry_field_job_repo.user_helper()
    }

    #[inline]
    pub(super) fn entry_field_job_repo(&self) -> &EntryFieldJobRepo {
        &self.entry_field_job_repo
    }

    #[inline]
    pub(super) fn entry_field_repo(&self) -> &EntryFieldRepo {
        self.entry_field_job_repo.entry_field_repo()
    }

    #[inline]
    pub(super) fn entry_repo(&self) -> &EntryRepo {
        self.entry_field_job_repo.entry_repo()
    }

    #[inline]
    pub(super) fn vernacular_repo(&self) -> &VernacularRepo {
        self.entry_field_job_repo.vernacular_repo()
    }

    #[inline]
    pub(super) fn entry_field_job_ttl(&self) -> Duration {
        self.entry_field_job_repo.entry_field_job_ttl()
    }

    #[inline]
    pub(super) fn expiry_from_time<Tz>(&self, time: DateTime<Tz>) -> DateTime<Tz>
    where
        Tz: TimeZone,
    {
        self.entry_field_job_repo.expiry_from_time(time)
    }

    #[inline]
    pub(super) fn expiry_from_now(&self) -> DateTime<Utc> {
        self.entry_field_job_repo.expiry_from_now()
    }

    pub async fn generate_active_filter(&self) -> Expr {
        let finished_expr = entry_field_job_attempts::Column::FinishedAt.is_not_null();
        let expiry_expr = entry_field_job_attempts::Column::ExpiresAt.gt(Utc::now());
        finished_expr.and(expiry_expr)
    }

    pub async fn generate_active_filter_for(&self, job_id: Id) -> Expr {
        let job_expr = entry_field_job_attempts::Column::JobId.eq(*job_id);
        job_expr.and(self.generate_active_filter().await)
    }

    pub async fn get_attempt<C>(
        &self,
        conn: &C,
        id: Id,
    ) -> Result<Option<entry_field_job_attempts::Model>, DcAppError>
    where
        C: ConnectionTrait + TransactionTrait,
    {
        entry_field_job_attempts::Entity::find_by_id(*id)
            .one(conn)
            .await
            .map_err(DcAppError::database_src)
    }

    pub async fn count_active_attempts_of<C>(&self, conn: &C, job_id: Id) -> Result<u64, DcAppError>
    where
        C: ConnectionTrait + TransactionTrait,
    {
        entry_field_job_attempts::Entity::find()
            .filter(self.generate_active_filter_for(job_id).await)
            .count(conn)
            .await
            .map_err(DcAppError::database_src)
    }

    async fn exists_active_attempts_of<C>(&self, conn: &C, job_id: Id) -> Result<bool, DcAppError>
    where
        C: ConnectionTrait,
    {
        entry_field_job_attempts::Entity::find()
            .filter(self.generate_active_filter_for(job_id).await)
            .exists(conn)
            .await
            .map_err(DcAppError::database_src)
    }

    async fn inner_create_attempt<C>(
        &self,
        conn: &C,
        job_id: Id,
        dto: CreateEntryFieldJobAttemptDto,
    ) -> Result<Option<entry_field_job_attempts::Model>, DcAppError>
    where
        C: ConnectionTrait,
    {
        let Some(job) = self
            .entry_field_job_repo
            .get_entry_field_job(conn, job_id.clone())
            .await?
        else {
            return Err(DcAppError::bad_state_desc(format!(
                "cannot create attempt for non-existing entry field job: {}",
                job_id
            )));
        };
        if self.exists_active_attempts_of(conn, job_id.clone()).await? {
            return Ok(None);
        }
        // TODO: replace with separate expiry config
        let am = dto.to_active_model(job, self.expiry_from_now())?;
        am.insert(conn)
            .await
            .map(Some)
            .map_err(DcAppError::database_src)
    }

    pub async fn create_attempt<C>(
        &self,
        conn: &C,
        job_id: Id,
        dto: CreateEntryFieldJobAttemptDto,
    ) -> Result<Option<entry_field_job_attempts::Model>, DcAppError>
    where
        C: ConnectionTrait + TransactionTrait,
    {
        let me = self.clone();
        conn.transaction(|txn| {
            Box::pin(async move { me.inner_create_attempt(txn, job_id, dto).await })
        })
        .await
        .map_err(DcAppError::flatten_tx)
    }

    async fn inner_finish_attempt<C>(
        &self,
        conn: &C,
        id: Id,
        dto: FinishEntryFieldJobAttemptDto,
    ) -> Result<Option<entry_field_job_attempts::Model>, DcAppError>
    where
        C: ConnectionTrait,
    {
        let Some(model) = entry_field_job_attempts::Entity::find_by_id(*id)
            .filter(self.generate_active_filter().await)
            .one(conn)
            .await
            .map_err(DcAppError::database_src)?
        else {
            return Ok(None);
        };
        let mut am = model.into_active_model();
        dto.update_active_model(&mut am)?;
        am.update(conn)
            .await
            .map(Some)
            .map_err(DcAppError::database_src)
    }

    pub async fn finish_attempt<C>(
        &self,
        conn: &C,
        id: Id,
        dto: FinishEntryFieldJobAttemptDto,
    ) -> Result<Option<entry_field_job_attempts::Model>, DcAppError>
    where
        C: ConnectionTrait + TransactionTrait,
    {
        let me = self.clone();
        conn.transaction(|txn| Box::pin(async move { me.inner_finish_attempt(txn, id, dto).await }))
            .await
            .map_err(DcAppError::flatten_tx)
    }

    async fn inner_delete_inactive_attempts<C>(
        &self,
        conn: &C,
        job_id: Id,
    ) -> Result<u64, DcAppError>
    where
        C: ConnectionTrait,
    {
        entry_field_job_attempts::Entity::delete_many()
            .filter(entry_field_job_attempts::Column::JobId.eq(*job_id))
            .filter(self.generate_active_filter().await.not())
            .exec(conn)
            .await
            .map(|r| r.rows_affected)
            .map_err(DcAppError::database_src)
    }

    pub async fn delete_inactive_attempts<C>(&self, conn: &C, job_id: Id) -> Result<u64, DcAppError>
    where
        C: ConnectionTrait + TransactionTrait,
    {
        let me = self.clone();
        conn.transaction(|txn| {
            Box::pin(async move { me.inner_delete_inactive_attempts(txn, job_id).await })
        })
        .await
        .map_err(DcAppError::flatten_tx)
    }
}
