use crate::drivers::cache::redis::{
    redis::{FromRedisValue, RedisError, ToRedisArgs},
    RedisConnection,
};
use async_trait::async_trait;
use redis::Commands;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Display;
use thiserror::Error;
use tracing::debug;

/// Implement on services that access Redis.
///
/// When implementing, the only necessary methods are the domain ,which simply returns a string
/// indicating the cache domain of implementing service and the connection. When implemented, the service will get
/// access to the `get`, `set`, `delete` and `get_json`, `set_json` utilities. This can be used by any
/// service component to get some ergonomics for manipulating the cache.
#[async_trait]
pub trait CacheAccess<Conn, K, T>
where
    K: Send + Sync,
    T: Send + Sync,
{
    async fn get(&self, key: &K) -> Result<T, CacheError>;

    async fn set(&self, key: &K, val: &T, ex: Option<usize>) -> Result<(), CacheError>;

    async fn delete(&self, key: &K) -> Result<(), CacheError>;
}

#[async_trait]
pub trait CacheAccessJson<Conn, K, T>
where
    K: Send + Sync,
    T: Serialize + DeserializeOwned + Send + Sync,
{
    async fn get_json(&self, key: &K) -> Result<T, CacheError>;

    async fn set_json(&self, key: &K, val: &T, ex: Option<usize>) -> Result<(), CacheError>;
}

pub trait Cacher {
    /// The first part of the cache key. Keys being cached by the implementing
    /// service will always be prefixed with whatever is returned from this method.
    fn domain() -> &'static str;

    /// Construct a full cache key using the domain, identifier and given key.
    /// Intended to be used by enums that serve as cache identifiers within a domain.
    fn construct_key<K: Display>(id: impl CacheIdentifier, key: K) -> String {
        format!("{}:{}:{}", Self::domain(), id.id(), key)
    }
}

/// Intended on use with enums to be used as cache identifiers,
/// i.e. the second part of the cache key after the domain
pub trait CacheIdentifier {
    fn id(self) -> &'static str;
}

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("Driver error {0}")]
    Driver(#[from] crate::drivers::DriverError),
    #[error("Redis error {0}")]
    Redis(#[from] RedisError),
    #[error("Serde error {0}")]
    Serde(#[from] serde_json::Error),
}
