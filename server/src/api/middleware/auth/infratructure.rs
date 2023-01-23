use super::contract::CacheContract;
use crate::{
    config::{cache::AuthCache, constants::SESSION_CACHE_DURATION_SECONDS},
    error::Error,
};
use chrono::Utc;
use infrastructure::{
    cache::CacheError,
    clients::redis::{Commands, Redis, RedisPoolConnection},
    CacheAccess,
};
use std::sync::Arc;
use storage::models::session::UserSession;

#[derive(Debug, Clone)]
pub struct Cache {
    pub client: Arc<Redis>,
}

impl CacheAccess for Cache {
    fn domain() -> &'static str {
        "auth"
    }

    fn connection(&self) -> Result<RedisPoolConnection, CacheError> {
        self.client.connect().map_err(|e| e.into())
    }
}

impl CacheContract for Cache {
    fn get_session_by_id(&self, id: &str) -> Result<UserSession, Error> {
        self.get_json(AuthCache::Session, id).map_err(Error::new)
    }

    fn cache_session(&self, id: &str, session: &UserSession) -> Result<(), Error> {
        self.set_json(
            AuthCache::Session,
            id,
            session,
            Some(SESSION_CACHE_DURATION_SECONDS),
        )
        .map_err(Error::new)
    }

    fn refresh_session(&self, session_id: &str) -> Result<(), Error> {
        let mut conn = self.client.connect()?;
        conn.expire_at(
            session_id,
            ((Utc::now().timestamp() + SESSION_CACHE_DURATION_SECONDS as i64) % i64::MAX) as usize,
        )?;
        Ok(())
    }
}
