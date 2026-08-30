use sea_orm::DatabaseConnection;
use tower_sessions::{SessionManagerLayer, service::CookieController};

use crate::{config::AuthConfig, error::AuthAppError, session::BoxedSessionStore};

/// Build the auth session [`SessionManagerLayer`] from the ORM-managed pool.
pub async fn build_session_layer(
    auth: &AuthConfig,
    db: &DatabaseConnection,
) -> Result<SessionManagerLayer<BoxedSessionStore, impl CookieController>, AuthAppError> {
    let store = BoxedSessionStore::store_from_sea(db).await?;
    let mut layer = SessionManagerLayer::new(store);
    layer = auth.configure_session_manager_layer_basics(layer);
    let signed_layer = auth.configure_signed_cookies(layer);
    Ok(signed_layer)
}
