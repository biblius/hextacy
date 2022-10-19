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
    models::session::Session,
};

pub(super) struct AuthenticationGuard {
    database: Postgres,
    cache: Cache,
}

impl AuthenticationGuard {
    pub(super) fn new(pg_pool: Arc<Pg>, rd_pool: Arc<Rd>) -> Self {
        Self {
            database: Postgres { pool: pg_pool },
            cache: Cache { pool: rd_pool },
        }
    }

    pub(super) async fn get_csrf_header(req: &ServiceRequest) -> Result<&str, Error> {
        req.headers().get("x-csrf-token").map_or_else(
            || Err(AuthenticationError::InvalidCsrfHeader.into()),
            |value| value.to_str().map_err(|e| e.into()),
        )
    }

    pub(super) async fn get_session_cookie(req: &ServiceRequest) -> Result<Cookie<'_>, Error> {
        req.cookie("session_id")
            .ok_or_else(|| AuthenticationError::SessionNotFound.into())
    }

    pub(super) async fn get_cached_session(&self, token: &str) -> Result<Session, Error> {
        self.cache.get_session_by_csrf(token).await
    }

    pub(super) async fn get_valid_session(
        &self,
        session_id: &str,
        csrf_token: &str,
    ) -> Result<Session, Error> {
        self.database
            .get_valid_session(session_id, csrf_token)
            .await
            .map_err(|_| AuthenticationError::SessionNotFound.into())
    }

    pub(super) async fn refresh_and_cache(
        &self,
        token: &str,
        session: &Session,
    ) -> Result<Session, Error> {
        let session = self.database.refresh_session(&session.id).await?;
        self.cache.cache_session(token, &session).await?;
        Ok(session)
    }
}

struct Postgres {
    pool: Arc<Pg>,
}

impl Postgres {
    async fn get_valid_session(&self, id: &str, csrf: &str) -> Result<Session, Error> {
        Session::get_valid_by_id(id, csrf, &mut self.pool.connect()?)
    }

    async fn refresh_session(&self, id: &str) -> Result<Session, Error> {
        Session::refresh(id, &mut self.pool.connect()?)?
            .pop()
            .ok_or_else(|| DatabaseError::DoesNotExist(format!("Session ID: {id}")).into())
    }
}

struct Cache {
    pool: Arc<Rd>,
}

impl Cache {
    async fn get_session_by_csrf(&self, token: &str) -> Result<Session, Error> {
        debug!(
            "Getting session under {}",
            format!("{}:{}", CacheId::Session, token)
        );
        Cacher::get(CacheId::Session, token, &mut self.pool.connect()?).map_err(|e| e.into())
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
        .map_err(|e| e.into())
    }
}
