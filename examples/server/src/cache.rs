pub mod adapters;
pub mod contracts;
pub mod driver;

use deadpool_redis::redis;
use std::fmt::Display;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CacheAdapterError {
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Serde error: {0}")]
    Serde(#[from] serde_json::Error),
}

pub trait Cacher {
    /// Construct a full cache key using the identifier and key.
    /// Intended to be used by enums that serve as cache identifiers.
    fn key<K: Display>(id: impl KeyPrefix, key: K) -> String {
        format!("{}:{}", id.id(), key)
    }
}

pub trait KeyPrefix {
    fn id(self) -> &'static str;
}

#[derive(Debug, PartialEq, Eq)]
pub enum TokenType {
    /// Keeps track of login attempts
    LoginAttempts,
    /// Session caching
    Session,
    /// Keeps track of registration tokens
    RegToken,
    /// Keeps track of password reset tokens
    PWToken,
    /// 2FA login, OTPs won't be accepted without this token in the cache
    OTPToken,
    /// 2FA login failure
    OTPThrottle,
    /// 2FA login failure
    OTPAttempts,
    /// Stopping email craziness
    EmailThrottle,
    /// Token obtained from a provider
    OAuth,
}

impl KeyPrefix for TokenType {
    fn id(self) -> &'static str {
        match self {
            Self::LoginAttempts => "auth:login_attempts",
            Self::Session => "auth:session",
            Self::RegToken => "auth:registration_token",
            Self::PWToken => "auth:set_pw",
            Self::OTPToken => "auth:otp",
            Self::OTPThrottle => "auth:otp_throttle",
            Self::OTPAttempts => "auth:otp_attempts",
            Self::EmailThrottle => "auth:emal_throttle",
            Self::OAuth => "auth:oauth",
        }
    }
}
