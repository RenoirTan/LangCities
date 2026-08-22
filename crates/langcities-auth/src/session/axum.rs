use sea_orm::DatabaseConnection;
use tower_sessions::{Expiry, SessionManagerLayer};

use crate::{config::AuthConfig, error::AuthAppError, session::BoxedSessionStore};

/// Build the auth session [`SessionManagerLayer`] from the ORM-managed pool.
pub async fn build_session_layer(
    _auth: &AuthConfig,
    db: &DatabaseConnection,
) -> Result<SessionManagerLayer<BoxedSessionStore>, AuthAppError> {
    let store = BoxedSessionStore::store_from_sea(db).await?;
    Ok(SessionManagerLayer::new(store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(time::Duration::days(1))))
}
