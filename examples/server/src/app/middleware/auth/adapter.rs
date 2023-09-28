use crate::cache::contracts::BasicCacheAccess;
use crate::cache::TokenType;
use crate::db::{models::session, repository::session::SessionRepository};
use crate::{config::constants::SESSION_CACHE_DURATION, error::Error};
use hextacy::{component, contract};

#[component(use Driver as driver, use SessionRepo)]
#[derive(Debug, Clone)]
pub struct AuthMwRepo {}

#[component(
    use D for Session: SessionRepository
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

#[component(use Driver as driver, use Cache)]
#[derive(Debug, Clone)]
pub struct AuthMwCache {}

#[component(
    use Driver for Cache: BasicCacheAccess
)]
#[contract]
impl AuthMwCache {
    async fn get_session_by_id(&self, id: &str) -> Result<session::Session, Error> {
        let mut conn = self.driver.connect().await?;
        Cache::get_json(&mut conn, TokenType::Session, id)
            .await
            .map_err(Error::new)
    }

    async fn cache_session(&self, id: &str, session: &session::Session) -> Result<(), Error> {
        let mut conn = self.driver.connect().await?;
        Cache::set_json(
            &mut conn,
            TokenType::Session,
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
            TokenType::Session,
            session_id,
            SESSION_CACHE_DURATION as i64,
        )
        .await
        .map_err(Error::new)
    }
}
