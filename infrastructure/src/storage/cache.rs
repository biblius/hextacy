use crate::clients::storage::redis::{Commands, RedisError, RedisPoolConnection, ToRedisArgs};
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Display;
use thiserror::Error;
use tracing::debug;

/// Implement on services that access the cache.
pub trait Cacher {
    /// The first part of the cache key. Keys being cached by the implementing
    /// service will always be prefixed with whatever is returned from this method.
    fn domain() -> &'static str;

    /// Construct a full cache key using the domain, identifier and given key.
    /// Intended to be used by enums that serve as cache identifiers within a domain.
    fn construct_key<K: ToRedisArgs + Display>(id: impl CacheIdentifier, key: K) -> String {
        format!("{}:{}:{}", Self::domain(), id.id(), key)
    }

    fn get<T: DeserializeOwned>(
        cache_id: impl CacheIdentifier,
        key: &str,
        conn: &mut RedisPoolConnection,
    ) -> Result<T, CacheError> {
        let key = Self::construct_key(cache_id, key);
        debug!("Getting {}", key);
        let result = conn.get::<&str, String>(&key)?;
        serde_json::from_str::<T>(&result).map_err(Into::into)
    }

    fn set<T: Serialize>(
        cache_id: impl CacheIdentifier,
        key: &str,
        val: &T,
        ex: Option<usize>,
        conn: &mut RedisPoolConnection,
    ) -> Result<(), CacheError> {
        let key = Self::construct_key(cache_id, key);
        debug!("Setting {}", key);
        let value = serde_json::to_string(&val)?;
        if let Some(ex) = ex {
            conn.set_ex::<&str, String, ()>(&key, value, ex)
                .map_err(Into::into)
        } else {
            conn.set::<&str, String, ()>(&key, value)
                .map_err(Into::into)
        }
    }

    fn delete(
        cache_id: impl CacheIdentifier,
        key: &str,
        conn: &mut RedisPoolConnection,
    ) -> Result<(), CacheError> {
        let key = Self::construct_key(cache_id, key);
        debug!("Deleting {}", key);
        conn.del::<String, ()>(key).map_err(Into::into)
    }
}

/// Intended on use with enums to be used as cache identifiers,
/// i.e. the second part of the cache key after the domain
pub trait CacheIdentifier {
    fn id(self) -> &'static str;
}

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("Redis error {0}")]
    Redis(#[from] RedisError),
    #[error("Serde error {0}")]
    Serde(#[from] serde_json::Error),
}
