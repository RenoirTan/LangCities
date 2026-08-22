use sea_orm::DatabaseConnection;

use crate::{config::Config, util::password::PasswordChecker};

#[derive(Clone, Debug)]
pub struct AppState {
    pub config: Config,
    pub db: DatabaseConnection,
    pub pw_checker: PasswordChecker,
}

impl AppState {
    pub fn new<C, D, P>(config: C, db: D, pw_checker: P) -> Self
    where
        C: Into<Config>,
        D: Into<DatabaseConnection>,
        P: Into<PasswordChecker>,
    {
        let (config, db, pw_checker) = (config.into(), db.into(), pw_checker.into());
        Self {
            config,
            db,
            pw_checker,
        }
    }
}
