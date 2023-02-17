use super::contract::CacheContract;
use crate::{
    config::{cache::AuthCache, constants::SESSION_CACHE_DURATION},
    error::Error,
};
use alx_core::{
    cache::CacheError,
    clients::db::redis::{Commands, Redis, RedisPoolConnection},
    CacheAccess,
};
use chrono::Utc;
use std::sync::Arc;
use storage::models::session::Session;

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
    fn get_session_by_id(&self, id: &str) -> Result<Session, Error> {
        self.get_json(AuthCache::Session, id).map_err(Error::new)
    }

    fn cache_session(&self, id: &str, session: &Session) -> Result<(), Error> {
        self.set_json(
            AuthCache::Session,
            id,
            session,
            Some(SESSION_CACHE_DURATION),
        )
        .map_err(Error::new)
    }

    fn refresh_session(&self, session_id: &str) -> Result<(), Error> {
        let mut conn = self.client.connect()?;
        conn.expire_at(
            session_id,
            ((Utc::now().timestamp() + SESSION_CACHE_DURATION as i64) % i64::MAX) as usize,
        )?;
        Ok(())
    }
}
