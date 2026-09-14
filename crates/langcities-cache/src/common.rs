use std::error::Error;

use langcities_config::datatype::Milliseconds;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Expiry {
    /// Keep the entry indefinitely, subject to normal cache eviction.
    None,
    /// Expire the entry this many milliseconds after it is set.
    Ttl(Milliseconds),
}

pub trait CacheBackend<K, V>: Send + Sync {
    fn has_key(&self, key: &K) -> impl Future<Output = bool> {
        async { self.get(key).await.is_some() }
    }

    fn get(&self, key: &K) -> impl Future<Output = Option<V>>;
    fn set(
        &self,
        key: K,
        value: V,
        expiry: Option<Expiry>,
    ) -> impl Future<Output = Result<Option<V>, Box<dyn Error + Send + Sync + 'static>>>;
    fn take(&self, key: &K) -> impl Future<Output = Option<V>>;

    fn delete(&self, key: &K) -> impl Future<Output = ()> {
        async {
            self.take(key).await;
        }
    }
}
