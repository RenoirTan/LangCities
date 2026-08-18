use sea_orm::DatabaseConnection;

#[derive(Clone, Debug)]
pub struct AppState {
    pub db: DatabaseConnection,
}

impl AppState {
    pub fn new<D>(db: D) -> Self
    where
        D: Into<DatabaseConnection>,
    {
        let db = db.into();
        Self { db }
    }
}
