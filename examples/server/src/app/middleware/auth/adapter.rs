use crate::cache::contracts::SimpleCacheAccess;
use crate::cache::AuthID;
use crate::db::{models::session, repository::session::SessionRepository};
use crate::{config::constants::SESSION_CACHE_DURATION, error::Error};
use hextacy::adapt;
use hextacy::contract;
use hextacy::drivers::Connect;

adapt! {
    AuthMwRepo,
    use D for Connection as driver;
    S: SessionRepository<Connection>,
}

#[contract]
impl<D, Connection, Session> AuthMwRepo<D, Connection, Session>
where
    Connection: Send,
    D: Connect<Connection = Connection> + Send + Sync,
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

adapt! {
    AuthMwCache,
    use Driver for Connection as driver;
    Cache: SimpleCacheAccess<Connection>
}

#[contract]
impl<D, Conn, Cache> AuthMwCache<D, Conn, Cache>
where
    Conn: Send,
    Cache: SimpleCacheAccess<Conn> + Send + Sync,
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

impl<D, C, S> Clone for AuthMwRepo<D, C, S>
where
    D: Connect<Connection = C> + Send + Sync,
    S: SessionRepository<C> + Send + Sync,
{
    fn clone(&self) -> Self {
        Self {
            driver: self.driver.clone(),
            ..*self
        }
    }
}

impl<D, C, Cache> Clone for AuthMwCache<D, C, Cache>
where
    D: Connect<Connection = C> + Send + Sync,
    Cache: SimpleCacheAccess<C> + Send + Sync,
{
    fn clone(&self) -> Self {
        Self {
            driver: self.driver.clone(),
            ..*self
        }
    }
}
