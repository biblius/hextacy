use crate::db::adapters::postgres::diesel::session::PgSessionAdapter;
use crate::db::{models::session, repository::session::SessionRepository};
use crate::{
    config::{cache::AuthCache, constants::SESSION_CACHE_DURATION},
    error::Error,
};
use chrono::Utc;
use hextacy::adapt;
use hextacy::drivers::cache::redis::{redis::Commands, Redis, RedisPoolConnection};
use hextacy::drivers::db::DBConnect;
use hextacy::service_component;
use hextacy::{
    cache::redis::CacheAccess,
    cache::redis::CacheError,
    drivers::db::postgres::{diesel::PgPoolConnection, diesel::PostgresDiesel},
};
use std::sync::Arc;

pub type Repo = Adapter<PostgresDiesel, PgPoolConnection, PgSessionAdapter>;

impl Clone for Repo {
    fn clone(&self) -> Self {
        Self {
            driver: self.driver.clone(),
            ..*self
        }
    }
}

adapt! {
    Adapter,

    use Diesel for Connection as driver: diesel;

    SessionRepository<Connection> as Session;

    async fn refresh_session(&self, id: &str, csrf: &str) -> Result<session::Session, Error> {
        let mut conn = self.driver.connect().await?;
        Session::refresh(&mut conn, id, csrf)
            .await
            .map_err(Error::new)
    }

    async fn get_valid_session(&self, id: &str, csrf: &str) -> Result<session::Session, Error> {
        let mut conn = self.driver.connect().await?;
        Session::get_valid_by_id(&mut conn, id, csrf)
            .await
            .map_err(Error::new)
    }

}

#[derive(Debug, Clone)]
pub struct Cache {
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

#[service_component(super)]
impl Cache {
    fn get_session_by_id(&self, id: &str) -> Result<session::Session, Error> {
        self.get_json(AuthCache::Session, id).map_err(Error::new)
    }

    fn cache_session(&self, id: &str, session: &session::Session) -> Result<(), Error> {
        self.set_json(
            AuthCache::Session,
            id,
            session,
            Some(SESSION_CACHE_DURATION),
        )
        .map_err(Error::new)
    }

    fn refresh_session(&self, session_id: &str) -> Result<(), Error> {
        let mut conn = self.driver.connect()?;
        conn.expire_at(
            session_id,
            ((Utc::now().timestamp() + SESSION_CACHE_DURATION as i64) % i64::MAX) as usize,
        )?;
        Ok(())
    }
}
