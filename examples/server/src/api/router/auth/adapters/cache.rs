use crate::config::cache::AuthCache;
use crate::config::constants::{
    EMAIL_THROTTLE_DURATION, OTP_THROTTLE_DURATION, SESSION_CACHE_DURATION,
    WRONG_PASSWORD_CACHE_DURATION,
};
use crate::db::models::session;
use crate::error::Error;
use chrono::Utc;
use hextacy::cache::redis::{CacheAccess, CacheError};
use hextacy::component;
use hextacy::drivers::cache::redis::RedisPoolConnection;
use hextacy::drivers::cache::redis::{redis::Commands, Redis};
use std::sync::Arc;
use tracing::debug;

pub(in crate::api::router::auth) struct Cache {
    pub driver: Arc<Redis>,
}

impl CacheAccess for Cache {
    fn domain() -> &'static str {
        "auth"
    }

    fn connection(&self) -> Result<RedisPoolConnection, CacheError> {
        self.driver.connect().map_err(|e| e.into())
    }
}

#[component(crate::api::router::auth)]
impl Cache {
    /// Sessions get cached behind the user's csrf token.
    fn set_session(&self, session_id: &str, session: &session::Session) -> Result<(), Error> {
        debug!("Caching session with ID {session_id}");
        self.set_json(
            AuthCache::Session,
            session_id,
            session,
            Some(SESSION_CACHE_DURATION),
        )
        .map_err(Error::new)
    }

    /// Sets a token as a key to the provided value in the cache
    fn set_token(
        &self,
        cache_id: AuthCache,
        token: &str,
        value: &str,
        ex: Option<usize>,
    ) -> Result<(), Error> {
        self.set(cache_id, token, value, ex).map_err(Error::new)
    }

    /// Gets a value from the cache stored under the token
    fn get_token(&self, cache_id: AuthCache, token: &str) -> Result<String, Error> {
        self.get(cache_id, token).map_err(Error::new)
    }

    /// Deletes the value in the cache stored under the token
    fn delete_token(&self, cache_id: AuthCache, token: &str) -> Result<(), Error> {
        self.delete(cache_id, token).map_err(Error::new)
    }

    /// Caches the number of login attempts using the user ID as the key. If the attempts do not exist they
    /// will be created, otherwise they will be incremented.
    fn cache_login_attempt(&self, user_id: &str) -> Result<u8, Error> {
        debug!("Caching login attempt for: {user_id}");
        let mut connection = self.driver.connect()?;
        let key = Self::construct_key(AuthCache::LoginAttempts, user_id);
        match connection.incr::<&str, u8, u8>(&key, 1) {
            Ok(c) => Ok(c),
            Err(_) => connection
                .set_ex::<String, u8, u8>(key, 1, WRONG_PASSWORD_CACHE_DURATION)
                .map_err(Error::new),
        }
    }

    /// Removes the user's login attempts from the cache
    fn delete_login_attempts(&self, user_id: &str) -> Result<(), Error> {
        debug!("Deleting login attempts for: {}", &user_id);
        self.delete(AuthCache::LoginAttempts, user_id)
            .map_err(Error::new)
    }

    fn get_otp_throttle(&self, cache_id: AuthCache, user_id: &str) -> Result<i64, Error> {
        self.get(cache_id, user_id).map_err(|e| e.into())
    }

    /// Cache the OTP throttle and attempts. The throttle always gets set to now and the attempts always get
    /// incremented. The domain should take care of the actual throttling.
    fn cache_otp_throttle(&self, user_id: &str) -> Result<i64, Error> {
        debug!("Throttling OTP attempts for: {user_id}");

        let mut connection = self.connection()?;

        let throttle_key = Self::construct_key(AuthCache::OTPThrottle, user_id);
        let attempt_key = Self::construct_key(AuthCache::OTPAttempts, user_id);

        match connection.get::<&str, i64>(&attempt_key) {
            Ok(attempts) => {
                // Override the throttle key to now
                connection
                    .set_ex::<&str, i64, _>(
                        &throttle_key,
                        Utc::now().timestamp(),
                        OTP_THROTTLE_DURATION,
                    )
                    .map_err(Error::new)?;

                // Increment the number of failed attempts
                connection
                    .set_ex::<&str, i64, _>(&attempt_key, attempts + 1, OTP_THROTTLE_DURATION)
                    .map_err(Error::new)?;
                Ok(attempts)
            }
            Err(_) => {
                // No key has been found in which case we cache
                connection
                    .set_ex::<&str, i64, _>(
                        &throttle_key,
                        Utc::now().timestamp(),
                        OTP_THROTTLE_DURATION,
                    )
                    .map_err(Error::new)?;
                connection
                    .set_ex::<&str, i64, _>(&attempt_key, 1, OTP_THROTTLE_DURATION)
                    .map_err(Error::new)
            }
        }
    }

    fn delete_otp_throttle(&self, user_id: &str) -> Result<(), Error> {
        self.delete(AuthCache::OTPThrottle, user_id)?;
        self.delete(AuthCache::OTPAttempts, user_id)?;
        Ok(())
    }

    fn set_email_throttle(&self, user_id: &str) -> Result<(), Error> {
        self.set(
            AuthCache::EmailThrottle,
            user_id,
            1,
            Some(EMAIL_THROTTLE_DURATION),
        )
        .map_err(|e| e.into())
    }

    fn get_email_throttle(&self, user_id: &str) -> Result<i64, Error> {
        self.get(AuthCache::EmailThrottle, user_id)
            .map_err(|e| e.into())
    }
}
