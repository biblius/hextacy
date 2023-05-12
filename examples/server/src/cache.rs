pub mod adapters;
pub mod contracts;

use hextacy::{cache::CacheError, drivers::cache::redis::redis};
use std::fmt::Display;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CacheAdapterError {
    #[error("Hextacy cache error: {0}")]
    Cache(#[from] CacheError),
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
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
pub enum AuthID {
    /// For keeping track of login attempts
    LoginAttempts,
    /// For caching sessions
    Session,
    /// For keeping track of registration tokens
    RegToken,
    /// For keeping track of password reset tokens
    PWToken,
    /// For 2FA login, OTPs won't be accepted without this token in the cache
    OTPToken,
    /// For 2FA login failure
    OTPThrottle,
    /// For 2FA login failure
    OTPAttempts,
    /// For stopping email craziness
    EmailThrottle,
}

impl KeyPrefix for AuthID {
    fn id(self) -> &'static str {
        use AuthID::*;
        match self {
            LoginAttempts => "auth:login_attempts",
            Session => "auth:session",
            RegToken => "auth:registration_token",
            PWToken => "auth:set_pw",
            OTPToken => "auth:otp",
            OTPThrottle => "auth:otp_throttle",
            OTPAttempts => "auth:otp_attempts",
            EmailThrottle => "auth:emal_throttle",
        }
    }
}
