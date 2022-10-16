use crate::{error::Error, models::session::Session};
use infrastructure::{
    config::constants::{SESSION_CACHE_DURATION_SECONDS, WRONG_PASSWORD_CACHE_DURATION},
    storage::redis::{Cache as Cacher, CacheId, Commands, Rd},
};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;
use tracing::debug;

pub(crate) struct Cache {
    pool: Arc<Rd>,
}

impl Cache {
    pub(crate) fn new(pool: Arc<Rd>) -> Self {
        Self { pool }
    }

    /// Sessions get cached behind the user's csrf token.
    pub(super) async fn set_session(
        &self,
        csrf_token: &str,
        session: &Session,
    ) -> Result<(), Error> {
        debug!("Caching session with token: {}", &csrf_token);
        let mut connection = self.pool.connect()?;

        Cacher::set(
            CacheId::Session,
            csrf_token,
            session,
            Some(SESSION_CACHE_DURATION_SECONDS),
            &mut connection,
        )?;

        Ok(())
    }

    /// Caches a user whenever they have 2fa enabled and attempt to login. Used to quickly fetch the user
    /// afterward to verify their otp password.
    pub(super) async fn set_token<T: Serialize>(
        &self,
        cache_id: CacheId,
        key: &str,
        value: &T,
        ex: Option<usize>,
    ) -> Result<(), Error> {
        debug!("Setting token: {} of type {}", &key, cache_id);
        let mut connection = self.pool.connect()?;
        Cacher::set(cache_id, key, value, ex, &mut connection).map_err(|e| e.into())
    }

    /// Fetches a cached user based on the provided otp token.
    pub(super) async fn get_token<T: DeserializeOwned>(
        &self,
        cache_id: CacheId,
        token: &str,
    ) -> Result<T, Error> {
        debug!("Fetching token: {} of type {}", &token, cache_id);
        let mut connection = self.pool.connect()?;
        Cacher::get(cache_id, token, &mut connection).map_err(|e| e.into())
    }

    /// Delete the cached otp token
    pub(super) async fn delete_token(&self, cache_id: CacheId, token: &str) -> Result<(), Error> {
        debug!("Deleting token: {} of type {}", &token, cache_id);
        let mut connection = self.pool.connect()?;
        Cacher::delete(cache_id, token, &mut connection).map_err(|e| e.into())
    }

    /// Caches the number of login attempts using the user ID as the key. If the attempts do not exist they
    /// will be created, otherwise they will be incremented.
    pub(super) async fn cache_login_attempt(&self, user_id: &str) -> Result<u8, Error> {
        let mut connection = self.pool.connect()?;
        debug!("Caching login attempt for: {}", &user_id);

        let key = Cacher::prefix_key(CacheId::LoginAttempts, &user_id);

        match connection.incr::<&str, u8, u8>(&key, 1) {
            Ok(c) => Ok(c),
            Err(_) => connection
                .set_ex::<String, u8, u8>(key, 1, WRONG_PASSWORD_CACHE_DURATION)
                .map_err(|e| e.into()),
        }
    }

    /// Removes the user's login attempts from the cache
    pub(super) async fn delete_login_attempts(&self, user_id: &str) -> Result<(), Error> {
        debug!("Deleting login attempts for: {}", &user_id);
        let mut connection = self.pool.connect()?;

        Cacher::delete(CacheId::LoginAttempts, user_id, &mut connection).map_err(|e| e.into())
    }
}
