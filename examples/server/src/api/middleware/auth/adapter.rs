use crate::db::adapters::postgres::seaorm::session::PgSessionAdapter;
use crate::db::{models::session, repository::session::SessionRepository};
use crate::{
    config::{cache::AuthCache as AuthCacheID, constants::SESSION_CACHE_DURATION},
    error::Error,
};
use async_trait::async_trait;
use chrono::Utc;
use hextacy::adapt;
use hextacy::contract;
use hextacy::drivers::cache::redis::{redis::Commands, Redis, RedisConnection};
use hextacy::drivers::{Connect, Driver};
use hextacy::{
    cache::CacheAccess,
    cache::CacheError,
    drivers::db::postgres::{seaorm::DatabaseConnection, seaorm::PostgresSea},
};
use std::sync::Arc;

pub type Repo = RepositoryComponent<PostgresSea, DatabaseConnection, PgSessionAdapter>;

impl Clone for Repo {
    fn clone(&self) -> Self {
        Self {
            driver: self.driver.clone(),
            ..*self
        }
    }
}

adapt! {
    RepositoryComponent,
    use D for Connection as driver;
    S: SessionRepository<Connection>,
}

#[contract]
impl<D, Connection, Session> RepositoryComponent<D, Connection, Session>
where
    D: Connect<Connection = Connection> + Send + Sync,
    Connection: Send,
    Session: SessionRepository<Connection> + Send + Sync,
{
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
pub struct AuthCache<D, Connection>
where
    D: Connect<Connection = Connection> + CacheAccess<Connection>,
{
    pub driver: Driver<D, Connection>,
}

/* #[async_trait]
impl CacheAccess<Conn> for AuthCache {
    fn domain() -> &'static str {
        "auth"
    }

    async fn connection(&self) -> Result<RedisConnection, CacheError> {
        self.driver.connect().await.map_err(|e| e.into())
    }
} */

#[contract(super)]
impl<D, Conn> AuthCache<D, Conn>
where
    D: Connect<Connection = Conn> + CacheAccess<Conn>,
{
    fn get_session_by_id(&self, id: &str) -> Result<session::Session, Error> {
        self.driver
            .get_json(AuthCache::Session, id)
            .map_err(Error::new)
    }

    fn cache_session(&self, id: &str, session: &session::Session) -> Result<(), Error> {
        self.driver
            .set_json(
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
