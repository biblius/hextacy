use crate::cache::contracts::SimpleCacheAccess;
use crate::cache::AuthID;
use crate::db::{models::session, repository::session::SessionRepository};
use crate::{config::constants::SESSION_CACHE_DURATION, error::Error};
use hextacy::drive;
use hextacy::Driver;
use hextacy::{component, contract};

drive! {
    AuthMwRepo,
    use D for Connection as driver;
    S: SessionRepository<Connection>,
}

#[component(
    use D for Connection,
    use SessionRepository with Connection as Session
)]
#[contract]
impl AuthMwRepo {
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

drive! {
    AuthMwCache,
    use Driver for Connection as driver;
    Cache: SimpleCacheAccess<Connection>
}

#[component(
    use D for Connection,
    use SimpleCacheAccess with Connection as Cache
)]
#[contract]
impl AuthMwCache {
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
    D: Driver<Connection = C> + Send + Sync + Clone,
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
    D: Driver<Connection = C> + Send + Sync + Clone,
    Cache: SimpleCacheAccess<C> + Send + Sync,
{
    fn clone(&self) -> Self {
        Self {
            driver: self.driver.clone(),
            ..*self
        }
    }
}
