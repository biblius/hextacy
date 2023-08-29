use crate::driver::{Driver, DriverError};
use async_trait::async_trait;
use std::{
    any::Any,
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
    sync::{Arc, Mutex},
};

type AnyHMap = HashMap<u64, Box<dyn Any + Send + Sync + 'static>>;

/// A simple, inefficient, but convenient implementation of a cache implementing the [Driver] trait.
/// Intended to be used during prototyping/testing. Do not use this in production as it is far from optimal.
///
/// Under the hood, the implementation uses hashes for the provided keys and
/// `Box<dyn Any>` for the values.
#[derive(Debug, Clone)]
pub struct InMemCache {
    pool: Arc<Mutex<AnyHMap>>,
}

impl Default for InMemCache {
    fn default() -> Self {
        Self {
            pool: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl InMemCache {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Contains a reference to the [InMemCache] this "connection" was
/// obtained from and provides a simple set of methods to manipulate the map.
#[derive(Debug, Clone)]
pub struct InMemConnection {
    cache: Arc<Mutex<AnyHMap>>,
}

impl InMemConnection {
    fn new(cache: &InMemCache) -> Self {
        Self {
            cache: cache.pool.clone(),
        }
    }

    pub fn set<K, V>(&mut self, key: K, value: V) -> Option<V>
    where
        K: Hash,
        V: Clone + Any + Send + Sync + 'static,
    {
        let mut map = self.cache.lock().unwrap();

        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hashed = hasher.finish();

        let res = map.insert(hashed, Box::new(value));

        res.map(|r| {
            *r.downcast::<V>()
                .expect("Invalid type provided for `value`")
        })
    }

    pub fn get<K, V>(&mut self, key: K) -> Option<V>
    where
        K: Hash,
        V: Clone + Any + Send + Sync + 'static,
    {
        let mut map = self.cache.lock().unwrap();

        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hashed = hasher.finish();

        let val = map.remove(&hashed)?;

        let actual = val
            .downcast::<V>()
            .expect("Invalid type provided for `value`");

        map.insert(hashed, actual.clone());

        Some(*actual)
    }

    pub fn remove<K, V>(&mut self, key: K) -> Option<V>
    where
        K: Hash,
        V: Clone + Any + Send + Sync + 'static,
    {
        let mut map = self.cache.lock().unwrap();

        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hashed = hasher.finish();

        map.remove(&hashed).map(|r| {
            *r.downcast::<V>()
                .expect("Invalid type provided for `value`")
        })
    }
}

#[async_trait]
impl Driver for InMemCache {
    type Connection = InMemConnection;

    async fn connect(&self) -> Result<Self::Connection, DriverError> {
        Ok(InMemConnection::new(self))
    }
}

#[cfg(test)]
mod tests {
    use crate::driver::Driver;

    use super::InMemCache;

    #[derive(Debug, Clone)]
    struct SomeItem {
        a: u8,
        b: &'static str,
    }

    #[tokio::test]
    async fn works() {
        let cache = InMemCache::new();
        let mut conn = cache.connect().await.unwrap();

        conn.set("ayy", "lmao");
        conn.set(
            3_usize,
            SomeItem {
                a: 1,
                b: "ayy lmao",
            },
        );

        assert_eq!(conn.cache.lock().unwrap().len(), 2);

        let ayy = conn.get::<&str, &str>("ayy").unwrap();
        let some_item = conn.get::<usize, SomeItem>(3).unwrap();

        assert_eq!(ayy, "lmao");
        assert_eq!(some_item.a, 1);
        assert_eq!(some_item.b, "ayy lmao");

        assert_eq!(conn.cache.lock().unwrap().len(), 2);

        conn.remove::<&str, &str>("ayy");
        conn.remove::<usize, SomeItem>(3);

        assert_eq!(conn.cache.lock().unwrap().len(), 0);
    }
}
