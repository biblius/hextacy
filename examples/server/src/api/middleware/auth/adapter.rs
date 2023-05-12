use std::marker::PhantomData;

use crate::cache::adapters::redis::RedisAdapter;
use crate::cache::contracts::AuthCacheAccess;
use crate::cache::AuthID;
use crate::db::adapters::postgres::seaorm::session::PgSessionAdapter;
use crate::db::{models::session, repository::session::SessionRepository};
use crate::{config::constants::SESSION_CACHE_DURATION, error::Error};
use async_trait::async_trait;
use chrono::Utc;
use hextacy::adapt;
use hextacy::contract;
use hextacy::drivers::cache::redis::redis::Commands;
use hextacy::drivers::cache::redis::{Redis, RedisConnection};
use hextacy::drivers::db::postgres::{seaorm::DatabaseConnection, seaorm::PostgresSea};
use hextacy::drivers::{Connect, Driver};

pub type Cache = CacheComponent<Redis, RedisConnection, RedisAdapter>;
pub type Repo = RepositoryComponent<PostgresSea, DatabaseConnection, PgSessionAdapter>;

impl Clone for Cache {
    fn clone(&self) -> Self {
        Self {
            driver: self.driver.clone(),
            ..*self
        }
    }
}

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

#[derive(Debug)]
pub struct CacheComponent<D, Connection, Cache>
where
    D: Connect<Connection = Connection>,
    Cache: AuthCacheAccess<Connection>,
{
    pub driver: Driver<D, Connection>,
    cache: PhantomData<Cache>,
}

#[contract]
impl<D, Conn, Cache> CacheComponent<D, Conn, Cache>
where
    Conn: Send,
    Cache: AuthCacheAccess<Conn> + Send + Sync,
    D: Connect<Connection = Conn> + Send + Sync,
{
    async fn get_session_by_id(&self, id: &str) -> Result<session::Session, Error> {
        let mut conn = self.driver.connect().await?;
        Cache::get_json(&mut conn, AuthID::Session, id)
            .await
            .map_err(Error::new)
    }

    async fn cache_session(&self, id: &str, session: &session::Session) -> Result<(), Error> {
        let mut conn = self.driver.connect().await?;
        Cache::set_json(
            &mut conn,
            AuthID::Session,
            id,
            session,
            Some(SESSION_CACHE_DURATION),
        )
        .await
        .map_err(Error::new)
    }

    async fn refresh_session(&self, session_id: &str) -> Result<(), Error> {
        let mut conn = self.driver.connect().await?;
        Cache::refresh(
            &mut conn,
            AuthID::Session,
            session_id,
            SESSION_CACHE_DURATION as i64,
        )
        .await
        .map_err(Error::new)
    }
}
