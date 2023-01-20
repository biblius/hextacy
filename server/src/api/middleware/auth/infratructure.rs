use super::contract::CacheContract;
use crate::{
    config::{cache::AuthCache, constants::SESSION_CACHE_DURATION_SECONDS},
    error::Error,
};
use chrono::Utc;
use infrastructure::{
    clients::storage::redis::{Commands, Redis},
    storage::{cache::Cacher, models::session::UserSession},
};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Cache {
    pub client: Arc<Redis>,
}

impl Cacher for Cache {
    fn domain() -> &'static str {
        "auth"
    }
}

impl CacheContract for Cache {
    fn get_session_by_id(&self, id: &str) -> Result<UserSession, Error> {
        <Self as Cacher>::get(AuthCache::Session, id, &mut self.client.connect()?)
            .map_err(Error::new)
    }

    fn cache_session(&self, id: &str, session: &UserSession) -> Result<(), Error> {
        <Self as Cacher>::set(
            AuthCache::Session,
            id,
            session,
            Some(SESSION_CACHE_DURATION_SECONDS),
            &mut self.client.connect()?,
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
