//! Server-side session cookie middleware built on `tower-sessions`.

use crate::{
    config::AuthConfig,
    error::{AuthAppError, AuthAppErrorTrait},
};
use sea_orm::{DatabaseConnection, DbBackend};
use tower_sessions::{
    Expiry, Session, SessionManagerLayer, SessionStore,
    session::{Id, Record},
};

#[cfg(feature = "mysql")]
use tower_sessions_sqlx_store::{MySqlStore, sqlx::MySqlPool};
#[cfg(feature = "postgres")]
use tower_sessions_sqlx_store::{PostgresStore, sqlx::PgPool};
#[cfg(feature = "sqlite")]
use tower_sessions_sqlx_store::{SqliteStore, sqlx::SqlitePool};

pub trait FullSessionStore: SessionStore {
    fn clone_box(&self) -> Box<dyn FullSessionStore>;
}

impl<T> FullSessionStore for T
where
    T: SessionStore + Clone + 'static,
{
    fn clone_box(&self) -> Box<dyn FullSessionStore> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn FullSessionStore> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[derive(Clone, Debug)]
pub struct BoxedSessionStore {
    inner: Box<dyn FullSessionStore>,
}

impl BoxedSessionStore {
    pub fn new(inner: impl FullSessionStore) -> Self {
        let inner = Box::new(inner) as Box<dyn FullSessionStore>;
        Self { inner }
    }
}

#[async_trait::async_trait]
impl SessionStore for BoxedSessionStore {
    async fn save(&self, session: &Record) -> Result<(), tower_sessions::session_store::Error> {
        self.inner.save(session).await
    }

    async fn load(
        &self,
        session_id: &Id,
    ) -> Result<Option<Record>, tower_sessions::session_store::Error> {
        self.inner.load(session_id).await
    }

    async fn delete(&self, session_id: &Id) -> Result<(), tower_sessions::session_store::Error> {
        self.inner.delete(session_id).await
    }
}

/// Retrieve the user id stored in a session.
pub async fn get_session_user_id<User>(session: &Session) -> Option<User>
where
    User: Send + Sync + for<'de> serde::Serialize + serde::de::DeserializeOwned + Clone,
{
    match session.get::<User>(USER_ID_FIELD_KEY).await {
        Ok(v) => v,
        Err(_) => None,
    }
}

const USER_ID_FIELD_KEY: &str = "user_id";

#[cfg(feature = "sqlite")]
async fn store_from_sqlite(db: &DatabaseConnection) -> Result<SqliteStore, AuthAppError> {
    let pool: SqlitePool = db.get_sqlite_connection_pool().clone();
    let store = SqliteStore::new(pool);
    store
        .migrate()
        .await
        .map_err(|e| AuthAppError::failed_session(Some(e.into())))?;
    Ok(store)
}

#[cfg(feature = "postgres")]
async fn store_from_postgres(db: &DatabaseConnection) -> Result<PostgresStore, AuthAppError> {
    let pool: PgPool = db.get_postgres_connection_pool().clone();
    let store = PostgresStore::new(pool);
    store
        .migrate()
        .await
        .map_err(|e| AuthAppError::failed_session(Some(e.into())))?;
    Ok(store)
}

#[cfg(feature = "mysql")]
async fn store_from_mysql(db: &DatabaseConnection) -> Result<MySqlStore, AuthAppError> {
    let pool: MySqlPool = db.get_mysql_connection_pool().clone();
    let store = MySqlStore::new(pool);
    store
        .migrate()
        .await
        .map_err(|e| AuthAppError::failed_session(Some(e.into())))?;
    Ok(store)
}

async fn store_from(db: &DatabaseConnection) -> Result<BoxedSessionStore, AuthAppError> {
    match db.get_database_backend() {
        #[cfg(feature = "sqlite")]
        DbBackend::Sqlite => store_from_sqlite(db)
            .await
            .map(|s| BoxedSessionStore::new(s)),
        #[cfg(feature = "mysql")]
        DbBackend::MySql => store_from_mysql(db)
            .await
            .map(|s| BoxedSessionStore::new(s)),
        #[cfg(feature = "postgres")]
        DbBackend::Postgres => store_from_postgres(db)
            .await
            .map(|s| BoxedSessionStore::new(s)),
        _ => Err(AuthAppError::other(None)),
    }
}

/// Build the auth session [`SessionManagerLayer`] from the ORM-managed pool.
pub async fn build_session_layer(
    _auth: &AuthConfig,
    db: &DatabaseConnection,
) -> Result<SessionManagerLayer<BoxedSessionStore>, AuthAppError> {
    let store = store_from(db).await?;
    Ok(SessionManagerLayer::new(store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(time::Duration::days(1))))
}
