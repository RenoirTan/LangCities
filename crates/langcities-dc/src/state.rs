use crate::{
    config::Config,
    error::{DcAppError, DcAppErrorTrait},
    repo::{
        entry::EntryRepo, entry_field::EntryFieldRepo,
        entry_field_dependency::EntryFieldDependencyRepo, entry_field_job::EntryFieldJobRepo,
        entry_field_job_attempt::EntryFieldJobAttemptRepo, user::UserRepo,
        vernacular::VernacularRepo,
    },
    util::user::UserHelper,
};
use sea_orm::{Database, DatabaseConnection};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: DatabaseConnection,
    pub user_helper: UserHelper,
}

impl AppState {
    pub fn new<C, D, U>(config: C, db: D, user_helper: U) -> Self
    where
        C: Into<Config>,
        D: Into<DatabaseConnection>,
        U: Into<UserHelper>,
    {
        let config = config.into();
        Self {
            config: Arc::new(config),
            db: db.into(),
            user_helper: user_helper.into(),
        }
    }

    pub async fn create<C>(config: C) -> Result<Self, DcAppError>
    where
        C: Into<Config>,
    {
        let config = config.into();
        let db = Database::connect(config.db.clone().to_connection_options())
            .await
            .map_err(DcAppError::database_src)?;
        let user_helper = UserHelper::create(&config)?;
        Ok(Self::new(config, db, user_helper))
    }

    pub fn user_repo(&self) -> UserRepo {
        UserRepo
    }

    pub fn vernacular_repo(&self) -> VernacularRepo {
        VernacularRepo::new(self.user_helper.clone())
    }

    pub fn entry_repo(&self) -> EntryRepo {
        EntryRepo::new(self.vernacular_repo())
    }

    pub fn entry_field_repo(&self) -> EntryFieldRepo {
        EntryFieldRepo::new(self.entry_repo())
    }

    pub fn entry_field_dependency_repo(&self) -> EntryFieldDependencyRepo {
        EntryFieldDependencyRepo::new(self.entry_field_repo())
    }

    pub fn entry_field_job_repo(&self) -> EntryFieldJobRepo {
        EntryFieldJobRepo::new(self.entry_field_repo(), self.config.dc.entry_field_job_ttl)
    }

    pub fn entry_field_job_attempt_repo(&self) -> EntryFieldJobAttemptRepo {
        EntryFieldJobAttemptRepo::new(self.entry_field_job_repo())
    }
}
