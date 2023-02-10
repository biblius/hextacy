use alx_clients::db::redis::{
    Commands, FromRedisValue, RedisError, RedisPoolConnection, ToRedisArgs,
};
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Display;
use thiserror::Error;
use tracing::debug;

/// Implement on services that access the cache.
///
/// When implementing, the only necessary methods are the domain ,which simply returns a string
/// indicating the cache domain of implementing service and the connection. When implemented, the service will get
/// access to the `get`, `set`, `delete` and `get_json`, `set_json` utilities. This can be used by any higher level
/// service to get some ergonomics for manipulating the cache.
pub trait CacheAccess {
    /// The first part of the cache key. Keys being cached by the implementing
    /// service will always be prefixed with whatever is returned from this method.
    fn domain() -> &'static str;

    fn connection(&self) -> Result<RedisPoolConnection, CacheError>;

    /// Construct a full cache key using the domain, identifier and given key.
    /// Intended to be used by enums that serve as cache identifiers within a domain.
    fn construct_key<K: ToRedisArgs + Display>(id: impl CacheIdentifier, key: K) -> String {
        format!("{}:{}:{}", Self::domain(), id.id(), key)
    }

    fn get<K: ToRedisArgs + Display, T: FromRedisValue>(
        &self,
        cache_id: impl CacheIdentifier,
        key: K,
    ) -> Result<T, CacheError> {
        let mut conn = self.connection()?;
        let key = Self::construct_key(cache_id, key);
        let result = conn.get::<&str, T>(&key)?;
        Ok(result)
    }

    fn set<T: ToRedisArgs>(
        &self,
        cache_id: impl CacheIdentifier,
        key: &str,
        val: T,
        ex: Option<usize>,
    ) -> Result<(), CacheError> {
        let mut conn = self.connection()?;
        let key = Self::construct_key(cache_id, key);
        debug!("Setting {}", key);
        if let Some(ex) = ex {
            conn.set_ex::<&str, T, ()>(&key, val, ex)
                .map_err(Into::into)
        } else {
            conn.set::<&str, T, ()>(&key, val).map_err(Into::into)
        }
    }

    fn get_json<T: DeserializeOwned>(
        &self,
        cache_id: impl CacheIdentifier,
        key: &str,
    ) -> Result<T, CacheError> {
        let mut conn = self.connection()?;
        let key = Self::construct_key(cache_id, key);
        debug!("Getting {}", key);
        let result = conn.get::<&str, String>(&key)?;
        serde_json::from_str::<T>(&result).map_err(Into::into)
    }

    fn set_json<T: Serialize>(
        &self,
        cache_id: impl CacheIdentifier,
        key: &str,
        val: &T,
        ex: Option<usize>,
    ) -> Result<(), CacheError> {
        let mut conn = self.connection()?;
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

    fn delete(&self, cache_id: impl CacheIdentifier, key: &str) -> Result<(), CacheError> {
        let mut conn = self.connection()?;
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
    #[error("Client error {0}")]
    Client(#[from] alx_clients::ClientError),
    #[error("Redis error {0}")]
    Redis(#[from] RedisError),
    #[error("Serde error {0}")]
    Serde(#[from] serde_json::Error),
}
