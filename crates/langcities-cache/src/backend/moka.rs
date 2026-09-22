use std::{
    hash::{BuildHasher, Hash, RandomState},
    time::Duration,
};

use langcities_common::error::Error;
use moka::{Expiry as MokaExpiry, future::Cache, ops::compute::Op};

use crate::common::{CacheBackend, Expiry};

#[derive(Clone, Debug)]
struct MokaValue<V> {
    value: V,
    expiry: Expiry,
}

struct MokaExpiryPolicy;

impl<K, V> MokaExpiry<K, MokaValue<V>> for MokaExpiryPolicy {
    fn expire_after_create(
        &self,
        _key: &K,
        value: &MokaValue<V>,
        _created_at: std::time::Instant,
    ) -> Option<Duration> {
        value.expiry.to_duration()
    }

    fn expire_after_update(
        &self,
        _key: &K,
        value: &MokaValue<V>,
        _updated_at: std::time::Instant,
        _duration_until_expiry: Option<Duration>,
    ) -> Option<Duration> {
        value.expiry.to_duration()
    }
}

#[derive(Clone, Debug)]
pub struct MokaWrapper<K, V, S = RandomState>
where
    K: Eq + Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
    S: BuildHasher + Clone + Send + Sync + 'static,
{
    inner: Cache<K, MokaValue<V>, S>,
}

impl<K, V, S> MokaWrapper<K, V, S>
where
    K: Eq + Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
    S: BuildHasher + Clone + Send + Sync + 'static,
{
    pub fn new_with_hasher(max_capacity: u64, hasher: S) -> Self {
        let inner = Cache::builder()
            .max_capacity(max_capacity)
            .expire_after(MokaExpiryPolicy)
            .build_with_hasher(hasher);
        Self { inner }
    }
}

impl<K, V> MokaWrapper<K, V>
where
    K: Eq + Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    pub fn new(max_capacity: u64) -> Self {
        Self::new_with_hasher(max_capacity, RandomState::default())
    }
}

impl<K, V, S> CacheBackend<K, V> for MokaWrapper<K, V, S>
where
    K: Eq + Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
    S: BuildHasher + Clone + Send + Sync + 'static,
{
    async fn has_key(&self, key: &K) -> bool {
        self.inner.contains_key(key)
    }

    async fn get(&self, key: &K) -> Option<V> {
        self.inner.get(key).await.map(|entry| entry.value)
    }

    async fn set(&self, key: K, value: V, expiry: Expiry) -> Result<Option<V>, Error> {
        let mut previous = None;
        self.inner
            .entry(key)
            .and_compute_with(|entry| async {
                if let Some(entry) = entry {
                    previous = Some(entry.into_value().value);
                }
                Op::Put(MokaValue { value, expiry })
            })
            .await;
        Ok(previous)
    }

    async fn take(&self, key: &K) -> Option<V> {
        self.inner.remove(key).await.map(|entry| entry.value)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::common::{CacheBackend, Expiry};

    use super::MokaWrapper;

    #[tokio::test]
    async fn ttl_expires_entries() {
        let cache = MokaWrapper::new(10);

        cache.set("key", "value", Expiry::Ttl(25)).await.unwrap();
        assert_eq!(cache.get(&"key").await, Some("value"));

        tokio::time::sleep(Duration::from_millis(50)).await;

        assert_eq!(cache.get(&"key").await, None);
    }

    #[tokio::test]
    async fn entries_without_expiry_remain_cached() {
        let cache = MokaWrapper::new(10);

        assert_eq!(
            cache.set("none", "value", Expiry::None).await.unwrap(),
            None
        );
        assert_eq!(
            cache
                .set("explicit-none", "value", Expiry::None)
                .await
                .unwrap(),
            None
        );

        tokio::time::sleep(Duration::from_millis(50)).await;

        assert_eq!(cache.get(&"none").await, Some("value"));
        assert_eq!(cache.get(&"explicit-none").await, Some("value"));
    }

    #[tokio::test]
    async fn updating_an_entry_replaces_its_expiry() {
        let cache = MokaWrapper::new(10);

        assert_eq!(
            cache.set("key", "short", Expiry::Ttl(100)).await.unwrap(),
            None
        );
        assert_eq!(
            cache.set("key", "long", Expiry::Ttl(200)).await.unwrap(),
            Some("short")
        );

        tokio::time::sleep(Duration::from_millis(120)).await;
        assert_eq!(cache.get(&"key").await, Some("long"));

        tokio::time::sleep(Duration::from_millis(100)).await;
        assert_eq!(cache.get(&"key").await, None);
    }

    #[tokio::test]
    async fn set_returns_the_previous_value() {
        let cache = MokaWrapper::new(10);

        assert_eq!(cache.set("key", "first", Expiry::None).await.unwrap(), None);
        assert_eq!(
            cache.set("key", "second", Expiry::None).await.unwrap(),
            Some("first")
        );
        assert_eq!(cache.get(&"key").await, Some("second"));
    }
}
