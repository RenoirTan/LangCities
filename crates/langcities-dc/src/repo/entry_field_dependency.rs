use langcities_lcdcdsl::component::Id;
use sea_orm::{
    ColumnTrait, Condition, ConnectionTrait, EntityTrait, QueryFilter, TransactionError,
    TransactionTrait, TryInsertResult,
};

use crate::{
    dto::entry_field_dependencies::{
        CreateEntryFieldDependencyDto, DeleteEntryFieldDependencyDto, ParentEntryFieldDependencyDto,
    },
    entity::entry_field_dependencies,
    error::{DcAppError, DcAppErrorTrait},
    repo::entry_field::EntryFieldRepo,
    state::AppState,
};

#[derive(Clone)]
pub struct EntryFieldDependencyRepo {
    pub entry_field_repo: EntryFieldRepo,
}

impl EntryFieldDependencyRepo {
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

    pub async fn get_dependencies_of<C>(
        &self,
        conn: &C,
        child_id: Id,
    ) -> Result<Vec<entry_field_dependencies::Model>, DcAppError>
    where
        C: ConnectionTrait,
    {
        entry_field_dependencies::Entity::find()
            .filter(entry_field_dependencies::Column::ChildEntryFieldId.eq(*child_id))
            .all(conn)
            .await
            .map_err(DcAppError::database)
    }

    pub async fn add_dependencies_of<C, D>(
        &self,
        conn: &C,
        child_id: Id,
        dependencies: D,
    ) -> Result<u64, DcAppError>
    where
        C: ConnectionTrait,
        D: IntoIterator<Item = CreateEntryFieldDependencyDto>,
    {
        let dependencies = dependencies
            .into_iter()
            .map(|d| d.to_active_model(child_id.clone()));
        let result = entry_field_dependencies::Entity::insert_many(dependencies)
            .on_conflict_do_nothing()
            .exec_without_returning(conn)
            .await
            .map_err(DcAppError::database)?;
        match result {
            TryInsertResult::Empty | TryInsertResult::Conflicted => Ok(0),
            TryInsertResult::Inserted(n) => Ok(n),
        }
    }

    pub async fn delete_dependencies_of<C, D>(
        &self,
        conn: &C,
        child_id: Id,
        dependencies: D,
    ) -> Result<u64, DcAppError>
    where
        C: ConnectionTrait,
        D: IntoIterator<Item = DeleteEntryFieldDependencyDto>,
    {
        let mut cond = Condition::any();
        for dep in dependencies {
            cond = cond.add(dep.to_expr());
        }
        let result = entry_field_dependencies::Entity::delete_many()
            .filter(entry_field_dependencies::Column::ChildEntryFieldId.eq(*child_id))
            .filter(cond)
            .exec(conn)
            .await
            .map_err(DcAppError::database)?;
        Ok(result.rows_affected)
    }

    pub async fn delete_all_dependencies_of<C>(
        &self,
        conn: &C,
        child_id: Id,
    ) -> Result<u64, DcAppError>
    where
        C: ConnectionTrait,
    {
        let result = entry_field_dependencies::Entity::delete_many()
            .filter(entry_field_dependencies::Column::ChildEntryFieldId.eq(*child_id))
            .exec(conn)
            .await
            .map_err(DcAppError::database)?;
        Ok(result.rows_affected)
    }

    async fn inner_sync_dependencies_of<C, D>(
        &self,
        conn: &C,
        child_id: Id,
        dependencies: D,
    ) -> Result<u64, DcAppError>
    where
        C: ConnectionTrait,
        D: IntoIterator<Item = ParentEntryFieldDependencyDto>,
    {
        self.delete_all_dependencies_of(conn, child_id.clone())
            .await?;
        self.add_dependencies_of(conn, child_id, dependencies).await
    }

    pub async fn sync_dependencies_of<C, D>(
        &self,
        conn: &C,
        child_id: Id,
        dependencies: D,
    ) -> Result<u64, DcAppError>
    where
        C: ConnectionTrait + TransactionTrait,
        D: IntoIterator<Item = ParentEntryFieldDependencyDto> + Send + 'static,
    {
        let me = self.clone();
        conn.transaction(|txn| {
            Box::pin(async move {
                me.inner_sync_dependencies_of(txn, child_id, dependencies)
                    .await
            })
        })
        .await
        .map_err(|e| match e {
            TransactionError::Connection(e) => DcAppError::database(e),
            TransactionError::Transaction(e) => e,
        })
    }
}
