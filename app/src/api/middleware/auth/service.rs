use std::sync::Arc;

use actix_web::{cookie::Cookie, dev::ServiceRequest};
use infrastructure::{
    config::constants::SESSION_CACHE_DURATION_SECONDS,
    storage::{
        postgres::Pg,
        redis::{Cache as Cacher, CacheId, Rd},
        DatabaseError,
    },
};
use tracing::debug;

use crate::{
    error::{AuthenticationError, Error},
    models::{role::Role, session::Session},
};

#[derive(Debug, Clone)]
pub(super) struct AuthenticationGuard {
    database: Postgres,
    cache: Cache,
    auth_level: Role,
}

impl AuthenticationGuard {
    pub(super) fn new(pg_pool: Arc<Pg>, rd_pool: Arc<Rd>, auth_level: Role) -> Self {
        Self {
            database: Postgres { pool: pg_pool },
            cache: Cache { pool: rd_pool },
            auth_level,
        }
    }

    /// Extracts the x-csrf-token header from the request
    pub(super) async fn get_csrf_header(req: &ServiceRequest) -> Result<&str, Error> {
        req.headers().get("x-csrf-token").map_or_else(
            || Err(AuthenticationError::InvalidCsrfHeader.into()),
            |value| value.to_str().map_err(Error::new),
        )
    }

    /// Extracts the `session_id` cookie from the request
    pub(super) async fn get_session_cookie(req: &ServiceRequest) -> Result<Cookie<'_>, Error> {
        req.cookie("session_id")
            .ok_or_else(|| AuthenticationError::Unauthenticated.into())
    }

    /// Attempts to obtain a session cached behind a csrf token
    pub(super) async fn get_cached_session(&self, token: &str) -> Result<Session, Error> {
        self.cache.get_session_by_csrf(token).await
    }

    /// Attempts to obtain a valid (unexpired) session corresponding to the user's csrf token
    pub(super) async fn get_valid_session(
        &self,
        session_id: &str,
        csrf_token: &str,
    ) -> Result<Session, Error> {
        self.database
            .get_valid_session(session_id, csrf_token)
            .await
            .map_err(|_| AuthenticationError::Unauthenticated.into())
    }

    /// Refreshes and caches the user session
    pub(super) async fn refresh_and_cache(
        &self,
        token: &str,
        session: &Session,
    ) -> Result<Session, Error> {
        let session = self.database.refresh_session(&session.id).await?;
        self.cache.cache_session(token, &session).await?;
        Ok(session)
    }

    /// Returns true if the role is equal to or greater than the auth_level of this guard instance
    #[inline]
    pub(super) fn check_valid_role(&self, role: &Role) -> bool {
        role >= &self.auth_level
    }
}

#[derive(Debug, Clone)]
struct Postgres {
    pool: Arc<Pg>,
}

impl Postgres {
    async fn get_valid_session(&self, id: &str, csrf: &str) -> Result<Session, Error> {
        debug!("Getting valid session with id {id} and csrf {csrf}");
        Session::get_valid_by_id(id, csrf, &mut self.pool.connect()?)
    }

    async fn refresh_session(&self, id: &str) -> Result<Session, Error> {
        debug!("Refreshing session with id {id}");
        Session::refresh(id, &mut self.pool.connect()?)?
            .pop()
            .ok_or_else(|| DatabaseError::DoesNotExist(format!("Session ID: {id}")).into())
    }
}

#[derive(Debug, Clone)]
struct Cache {
    pool: Arc<Rd>,
}

impl Cache {
    async fn get_session_by_csrf(&self, token: &str) -> Result<Session, Error> {
        debug!(
            "Getting session under {}",
            format!("{}:{}", CacheId::Session, token)
        );
        Cacher::get(CacheId::Session, token, &mut self.pool.connect()?).map_err(Error::new)
    }

    async fn cache_session(&self, token: &str, session: &Session) -> Result<(), Error> {
        debug!(
            "Caching session under {}",
            format!("{}:{}", CacheId::Session, token)
        );
        Cacher::set(
            CacheId::Session,
            token,
            session,
            Some(SESSION_CACHE_DURATION_SECONDS),
            &mut self.pool.connect()?,
        )
        .map_err(Error::new)
    }
}
