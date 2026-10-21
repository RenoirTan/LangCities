use std::{borrow::Borrow, collections::HashMap};

use langcities_cache::{
    backend::moka::MokaWrapper,
    common::{CacheBackend, Expiry},
};
use langcities_common::error::Error;
use langcities_lcdcdsl::component::{Id, Slug};
use langcities_server::{auth_client::AuthClient, dto::users::AuthUserDto};

use crate::{
    config::Config,
    error::{DcAppError, DcAppErrorTrait},
};

pub struct UserHelper {
    auth_client: AuthClient,
    username_cache: Box<dyn CacheBackend<Slug, Id>>,
    expiry: Expiry,
}

impl UserHelper {
    pub fn new<A, U, E>(auth_client: A, username_cache: U, expiry: E) -> Self
    where
        A: Into<AuthClient>,
        U: Into<Box<dyn CacheBackend<Slug, Id>>>,
        E: Into<Expiry>,
    {
        let (auth_client, username_cache, expiry) =
            (auth_client.into(), username_cache.into(), expiry.into());
        Self {
            auth_client,
            username_cache,
            expiry,
        }
    }

    pub fn create<C>(config: &C) -> Result<Self, DcAppError>
    where
        C: Borrow<Config>,
    {
        let config = config.borrow();
        let username_cache = MokaWrapper::new(config.dc.username_cache_max_capacity);
        let username_cache = Box::new(username_cache) as Box<dyn CacheBackend<Slug, Id>>;
        let auth_client =
            AuthClient::new(&config.dc.auth_base_url).map_err(DcAppError::failed_init_src)?;
        let expiry = config.dc.username_cache_expiry.clone();
        Ok(Self::new(auth_client, username_cache, expiry))
    }

    pub async fn set_username(
        &self,
        username: Slug,
        auth_user_id: Id,
    ) -> Result<Option<Id>, Error> {
        self.username_cache
            .set(username, auth_user_id, self.expiry.clone())
            .await
    }

    pub async fn get_cached_id_from_username(&self, username: &Slug) -> Option<Id> {
        self.username_cache.get(username).await
    }

    pub async fn get_cached_ids_from_usernames(&self, usernames: &[&Slug]) -> HashMap<Slug, Id> {
        self.username_cache
            .get_many(&mut usernames.iter().map(|u| *u))
            .await
    }

    pub async fn fetch_and_cache_aliases(
        &self,
        aliases: &[&Slug],
    ) -> Result<Vec<AuthUserDto>, DcAppError> {
        let users = self
            .auth_client
            .fetch_users(aliases.iter().map(|a| *a))
            .await
            .map_err(DcAppError::auth_service_src)?;

        for user in &users {
            self.set_username(user.username.clone(), user.id.clone())
                .await
                .map_err(DcAppError::cache_src)?;
        }
        Ok(users)
    }

    pub async fn resolve_auth_user_id<S>(&self, username: &S) -> Result<Id, DcAppError>
    where
        S: Borrow<Slug>,
    {
        let username = username.borrow();
        if let Some(id) = self.get_cached_id_from_username(username).await {
            return Ok(id);
        }
        let users = self
            .fetch_and_cache_aliases(std::slice::from_ref(&username))
            .await?;
        // Use the response directly: a cache entry can expire or be evicted immediately.
        users
            .into_iter()
            .find(|user| user.username == *username)
            .map(|user| user.id)
            .ok_or_else(|| DcAppError::not_found_desc(format!("user '{username}' not found")))
    }

    pub async fn resolve_auth_user_ids(
        &self,
        usernames: &[&Slug],
    ) -> Result<HashMap<Slug, Id>, DcAppError> {
        let map = self.get_cached_ids_from_usernames(usernames).await;
        let missing_usernames = usernames
            .iter()
            .map(|u| *u)
            .filter(|u| map.contains_key(*u))
            .collect::<Vec<_>>();
        if missing_usernames.len() <= 0 {
            return Ok(map);
        }
        let users = self.fetch_and_cache_aliases(&missing_usernames).await?;
        let map = users
            .into_iter()
            .map(|d| (d.username, d.id))
            .collect::<HashMap<_, _>>();
        for username in usernames.iter().map(|u| *u) {
            if !map.contains_key(username) {
                return Err(DcAppError::cache_desc(format!("{username} not found")));
            }
        }
        Ok(map)
    }
}

impl Clone for UserHelper {
    fn clone(&self) -> Self {
        Self {
            auth_client: self.auth_client.clone(),
            username_cache: self.username_cache.cloned(),
            expiry: self.expiry.clone(),
        }
    }
}
