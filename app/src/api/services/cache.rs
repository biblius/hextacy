use crate::error::Error;
use infrastructure::storage::redis::{Commands, RedisPoolConnection, ToRedisArgs};
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Display;

pub struct Cache;

impl Cache {
    pub fn get<T: DeserializeOwned>(
        prefix: CachePrefix,
        key: &str,
        conn: &mut RedisPoolConnection,
    ) -> Result<T, Error> {
        let result = conn.get::<&str, String>(key)?;
        serde_json::from_str::<T>(&result).map_err(|e| e.into())
    }

    pub fn set<T: Serialize>(
        prefix: CachePrefix,
        key: &str,
        val: &T,
        ex: Option<usize>,
        conn: &mut RedisPoolConnection,
    ) -> Result<(), Error> {
        let key = Self::prefix_key(prefix, &key);
        let value = serde_json::to_string(&val)?;
        if let Some(ex) = ex {
            conn.set_ex::<&str, String, ()>(&key, value, ex)
                .map_err(|e| e.into())
        } else {
            conn.set::<&str, String, ()>(&key, value)
                .map_err(|e| e.into())
        }
    }

    fn prefix_key<T: ToRedisArgs + Display>(prefix: CachePrefix, key: &T) -> String {
        format!("{}:{}", prefix, key)
    }
}

pub enum CachePrefix {
    LoginAttempts(String),
    TempOtp,
    CSRF,
    Session,
}

impl Display for CachePrefix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CachePrefix::LoginAttempts(user_id) => write!(f, "auth:login_attempts:{}", user_id),
            CachePrefix::CSRF => write!(f, "auth:csrf"),
            CachePrefix::TempOtp => write!(f, "auth:temp_otp"),
            CachePrefix::Session => write!(f, "auth:session"),
        }
    }
}
