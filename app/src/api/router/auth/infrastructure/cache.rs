use crate::{error::Error, models::session::Session};
use infrastructure::{
    config::constants::{SESSION_CACHE_DURATION_SECONDS, WRONG_PASSWORD_CACHE_DURATION},
    storage::redis::{Cache as Cacher, CacheId, Commands, Rd},
};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;
use tracing::debug;

pub(in super::super) struct Cache {
    pool: Arc<Rd>,
}

impl Cache {
    pub(in super::super) fn new(pool: Arc<Rd>) -> Self {
        Self { pool }
    }

    /// Sessions get cached behind the user's csrf token.
    pub(in super::super) async fn set_session(
        &self,
        csrf_token: &str,
        session: &Session,
    ) -> Result<(), Error> {
        let mut connection = self.pool.connect()?;
        Cacher::set(
            CacheId::Session,
            csrf_token,
            session,
            Some(SESSION_CACHE_DURATION_SECONDS),
            &mut connection,
        )
        .map_err(Error::new)
    }

    /// Caches a user whenever they have 2fa enabled and attempt to login. Used to quickly fetch the user
    /// afterward to verify their otp password.
    pub(in super::super) async fn set_token<T: Serialize>(
        &self,
        cache_id: CacheId,
        key: &str,
        value: &T,
        ex: Option<usize>,
    ) -> Result<(), Error> {
        let mut connection = self.pool.connect()?;
        Cacher::set(cache_id, key, value, ex, &mut connection).map_err(Error::new)
    }

    /// Fetches a cached user based on the provided otp token.
    pub(in super::super) async fn get_token<T: DeserializeOwned>(
        &self,
        cache_id: CacheId,
        token: &str,
    ) -> Result<T, Error> {
        let mut connection = self.pool.connect()?;
        Cacher::get(cache_id, token, &mut connection).map_err(Error::new)
    }

    /// Delete the cached otp token
    pub(in super::super) async fn delete_token(
        &self,
        cache_id: CacheId,
        token: &str,
    ) -> Result<(), Error> {
        let mut connection = self.pool.connect()?;
        Cacher::delete(cache_id, token, &mut connection).map_err(Error::new)
    }

    /// Caches the number of login attempts using the user ID as the key. If the attempts do not exist they
    /// will be created, otherwise they will be incremented.
    pub(in super::super) async fn cache_login_attempt(&self, user_id: &str) -> Result<u8, Error> {
        let mut connection = self.pool.connect()?;
        let key = Cacher::prefix_id(CacheId::LoginAttempts, &user_id);
        match connection.incr::<&str, u8, u8>(&key, 1) {
            Ok(c) => Ok(c),
            Err(_) => connection
                .set_ex::<String, u8, u8>(key, 1, WRONG_PASSWORD_CACHE_DURATION)
                .map_err(Error::new),
        }
    }

    /// Removes the user's login attempts from the cache
    pub(in super::super) async fn delete_login_attempts(&self, user_id: &str) -> Result<(), Error> {
        debug!("Deleting login attempts for: {}", &user_id);
        let mut connection = self.pool.connect()?;
        Cacher::delete(CacheId::LoginAttempts, user_id, &mut connection).map_err(Error::new)
    }
}
