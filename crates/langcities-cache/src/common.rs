use std::{collections::HashMap, time::Duration};

use async_trait::async_trait;
use langcities_common::error::Error;
use langcities_config::datatype::Milliseconds;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Expiry {
    /// Keep the entry indefinitely, subject to normal cache eviction.
    None,
    /// Expire the entry this many milliseconds after it is set.
    Ttl(Milliseconds),
}

impl Expiry {
    pub fn to_duration(&self) -> Option<Duration> {
        match self {
            Self::None => None,
            Self::Ttl(ms) => Some(Duration::from_millis(*ms)),
        }
    }
}

#[async_trait]
pub trait CacheBackend<K, V>: Send + Sync
where
    K: Sync,
{
    fn cloned(&self) -> Box<dyn CacheBackend<K, V>>;

    async fn has_key(&self, key: &K) -> bool {
        self.get(key).await.is_some()
    }

    async fn get(&self, key: &K) -> Option<V>;
    async fn get_many(&self, keys: &[&K]) -> HashMap<K, V>;
    async fn set(&self, key: K, value: V, expiry: Expiry) -> Result<Option<V>, Error>;
    async fn take(&self, key: &K) -> Option<V>;

    async fn delete(&self, key: &K) -> () {
        self.take(key).await;
    }
}
