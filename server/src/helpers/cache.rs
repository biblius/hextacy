use infrastructure::clients::store::redis::{
    Commands, RedisError, RedisPoolConnection, ToRedisArgs,
};
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Display;
use thiserror::Error;
use tracing::debug;

/// Utility struct
pub struct Cache;

impl Cache {
    pub fn get<T: DeserializeOwned>(
        cache_id: CacheId,
        key: &str,
        conn: &mut RedisPoolConnection,
    ) -> Result<T, CacheError> {
        debug!("Getting {}:{}", cache_id, key);
        let key = Self::prefix_id(cache_id, &key);
        let result = conn.get::<&str, String>(&key)?;
        serde_json::from_str::<T>(&result).map_err(Into::into)
    }

    pub fn set<T: Serialize>(
        cache_id: CacheId,
        key: &str,
        val: &T,
        ex: Option<usize>,
        conn: &mut RedisPoolConnection,
    ) -> Result<(), CacheError> {
        debug!("Setting {}:{}", cache_id, key);
        let key = Self::prefix_id(cache_id, &key);
        let value = serde_json::to_string(&val)?;
        if let Some(ex) = ex {
            conn.set_ex::<&str, String, ()>(&key, value, ex)
                .map_err(Into::into)
        } else {
            conn.set::<&str, String, ()>(&key, value)
                .map_err(Into::into)
        }
    }

    pub fn delete(
        cache_id: CacheId,
        key: &str,
        conn: &mut RedisPoolConnection,
    ) -> Result<(), CacheError> {
        debug!("Deleting {}:{}", cache_id, key);
        conn.del::<String, ()>(Self::prefix_id(cache_id, &key))
            .map_err(Into::into)
    }

    pub fn prefix_id<T: ToRedisArgs + Display>(cache_id: CacheId, key: &T) -> String {
        format!("{}:{}", cache_id, key)
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
            CacheId::LoginAttempts => write!(f, "auth:login_attempts"),
            CacheId::OTPToken => write!(f, "auth:otp"),
            CacheId::OTPThrottle => write!(f, "auth:otp_throttle"),
            CacheId::OTPAttempts => write!(f, "auth:otp_attempts"),
            CacheId::Session => write!(f, "auth:session"),
            CacheId::RegToken => write!(f, "auth:registration_token"),
            CacheId::PWToken => write!(f, "auth:set_pw"),
            CacheId::EmailThrottle => write!(f, "auth:email_throttle"),
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
