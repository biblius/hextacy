use crate::clients::storage::redis::{Commands, RedisError, RedisPoolConnection, ToRedisArgs};
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Display;
use thiserror::Error;
use tracing::debug;

pub struct Cache {}

/// Implement on services that access the cache.
pub trait Cacher {
    /// The first part of the cache key. Keys being cached by the implementing
    /// service will always be prefixed with whatever is returned from this method.
    fn domain() -> &'static str;

    /// Construct a full cache key using the domain, identifier and given key.
    /// Intended to be used by enums that serve as cache identifiers within a domain.
    fn construct_key<K: ToRedisArgs + Display>(id: impl Display, key: K) -> String {
        format!("{}:{}:{}", Self::domain(), id, key)
    }

    fn get<T: DeserializeOwned>(
        cache_id: CacheId,
        key: &str,
        conn: &mut RedisPoolConnection,
    ) -> Result<T, CacheError> {
        debug!("Getting {}:{}", cache_id, key);
        let key = Self::construct_key(cache_id, key);
        let result = conn.get::<&str, String>(&key)?;
        serde_json::from_str::<T>(&result).map_err(Into::into)
    }

    fn set<T: Serialize>(
        cache_id: CacheId,
        key: &str,
        val: &T,
        ex: Option<usize>,
        conn: &mut RedisPoolConnection,
    ) -> Result<(), CacheError> {
        debug!("Setting {}:{}", cache_id, key);
        let key = Self::construct_key(cache_id, key);
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
        cache_id: CacheId,
        key: &str,
        conn: &mut RedisPoolConnection,
    ) -> Result<(), CacheError> {
        debug!("Deleting {}:{}", cache_id, key);
        conn.del::<String, ()>(Self::construct_key(cache_id, key))
            .map_err(Into::into)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum CacheId {
    /// For keeping track of login attempts
    LoginAttempts,
    /// For caching permanent sessions
    Session,
    /// For keeping track of registration tokens
    RegToken,
    /// For keeping track of password reset tokens
    PWToken,
    /// For 2FA login, OTPs won't be accepted without this token in the cache
    OTPToken,
    /// For 2FA login failure
    OTPThrottle,
    OTPAttempts,
    /// For stopping email craziness
    EmailThrottle,
}

impl Display for CacheId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheId::LoginAttempts => write!(f, "login_attempts"),
            CacheId::OTPToken => write!(f, "otp"),
            CacheId::OTPThrottle => write!(f, "otp_throttle"),
            CacheId::OTPAttempts => write!(f, "otp_attempts"),
            CacheId::Session => write!(f, "session"),
            CacheId::RegToken => write!(f, "registration_token"),
            CacheId::PWToken => write!(f, "set_pw"),
            CacheId::EmailThrottle => write!(f, "email_throttle"),
        }
    }
}

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("Redis error {0}")]
    Redis(#[from] RedisError),
    #[error("Serde error {0}")]
    Serde(#[from] serde_json::Error),
}
