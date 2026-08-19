use sea_orm::DatabaseConnection;

use crate::util::password::PasswordChecker;

#[derive(Clone, Debug)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub pw_checker: PasswordChecker,
}

impl AppState {
    pub fn new<D, P>(db: D, pw_checker: P) -> Self
    where
        D: Into<DatabaseConnection>,
        P: Into<PasswordChecker>,
    {
        let (db, pw_checker) = (db.into(), pw_checker.into());
        Self { db, pw_checker }
    }
}
