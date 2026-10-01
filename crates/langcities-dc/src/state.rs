use crate::{
    config::Config,
    error::{DcAppError, DcAppErrorTrait},
};
use langcities_cache::{backend::moka::MokaWrapper, common::CacheBackend};
use langcities_common::error::Error;
use langcities_common_server::{auth_client::AuthClient, dto::users::AuthUserDto};
use sea_orm::{Database, DatabaseConnection};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: DatabaseConnection,
    pub auth_client: AuthClient,
    /// Maps usernames to auth user IDs (not DC user IDs).
    /// Cheap to clone because internally it all points to the same RC-ed inner data
    pub username_cache: MokaWrapper<String, i64>,
}

impl AppState {
    pub fn new<C, D>(config: C, db: D) -> Result<Self, DcAppError>
    where
        C: Into<Config>,
        D: Into<DatabaseConnection>,
    {
        let config = config.into();
        let username_cache = MokaWrapper::new(config.dc.username_cache_max_capacity);
        let auth_client =
            AuthClient::new(&config.dc.auth_base_url).map_err(DcAppError::failed_init)?;
        Ok(Self {
            config: Arc::new(config),
            db: db.into(),
            username_cache,
            auth_client,
        })
    }

    pub async fn set_username_cache(
        &self,
        username: String,
        id: i64,
    ) -> Result<Option<i64>, Error> {
        let expiry = self.config.dc.username_cache_expiry.clone();
        self.username_cache.set(username, id, expiry).await
    }

    pub async fn get_cached_id_from_username(&self, username: &String) -> Option<i64> {
        self.username_cache.get(username).await
    }

    pub async fn fetch_and_cache_auth_users(
        &self,
        aliases: &[&str],
    ) -> Result<Vec<AuthUserDto>, DcAppError> {
        let users = self
            .auth_client
            .fetch_users(aliases)
            .await
            .map_err(DcAppError::auth_service)?;

        for user in &users {
            self.set_username_cache(user.username.clone(), user.id)
                .await
                .map_err(DcAppError::cache)?;
        }
        Ok(users)
    }

    pub async fn resolve_auth_user_id(&self, username: &str) -> Result<i64, DcAppError> {
        if let Some(id) = self
            .get_cached_id_from_username(&username.to_string())
            .await
        {
            return Ok(id);
        }
        let users = self
            .fetch_and_cache_auth_users(std::slice::from_ref(&username))
            .await?;
        // Use the response directly: a cache entry can expire or be evicted immediately.
        users
            .into_iter()
            .find(|user| user.username == username)
            .map(|user| user.id)
            .ok_or_else(|| DcAppError::not_found(format!("user '{username}' not found")))
    }

    pub async fn create<C>(config: C) -> Result<Self, DcAppError>
    where
        C: Into<Config>,
    {
        let config = config.into();
        let db = Database::connect(config.db.clone().to_connection_options())
            .await
            .map_err(DcAppError::database)?;
        Self::new(config, db)
    }
}
