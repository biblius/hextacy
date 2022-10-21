use std::sync::Arc;

use actix_web::{cookie::Cookie, dev::ServiceRequest};
use infrastructure::{
    config::constants::SESSION_CACHE_DURATION_SECONDS,
    http::cookie::S_ID,
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

    /// Attempts to get a session from the cache. If it doesn't exist, checks the database for an unexpired session.
    /// Then if the session is found and permanent, caches it. If it's not permanent, refreshes it for 30 minutes.
    /// If it can't find a session returns an `Unauthenticated` error.
    pub(super) async fn process_session(
        &self,
        session_id: &str,
        csrf: &str,
    ) -> Result<Session, Error> {
        // Check cache
        if let Ok(session) = self.cache.get_session_by_csrf(csrf).await {
            debug!("Found cached session with {session_id}");
            Ok(session)
        } else {
            // Check DB
            if let Ok(session) = self.database.get_valid_session(session_id, csrf).await {
                debug!("Found valid session with id {}", session.id);

                // Cache if permanent
                if session.is_permanent() {
                    debug!("Session permanent, caching {}", session.id);
                    self.cache.cache_session(csrf, &session).await?;
                    return Ok(session);
                }

                // Otherwise refresh
                debug!("Refreshing session {}", session.id);
                self.database.refresh_session(&session.id).await?;
                Ok(session)
            } else {
                Err(Error::new(AuthenticationError::Unauthenticated))
            }
        }
    }

    /// Extracts the x-csrf-token header from the request
    pub(super) async fn get_csrf_header(req: &ServiceRequest) -> Result<&str, Error> {
        req.headers().get("x-csrf-token").map_or_else(
            || Err(AuthenticationError::InvalidCsrfHeader.into()),
            |value| value.to_str().map_err(Error::new),
        )
    }

    /// Extracts the `S_ID` cookie from the request
    pub(super) async fn get_session_cookie(req: &ServiceRequest) -> Result<Cookie<'_>, Error> {
        req.cookie(S_ID)
            .ok_or_else(|| AuthenticationError::Unauthenticated.into())
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
    /// Attempts to find an unexpired session with its corresponding CSRF
    async fn get_valid_session(&self, id: &str, csrf: &str) -> Result<Session, Error> {
        Session::get_valid_by_id(id, csrf, &mut self.pool.connect()?)
    }

    /// Extends session `expires_at` for 30 minutes
    async fn refresh_session(&self, id: &str) -> Result<Session, Error> {
        Session::refresh_temporary(id, &mut self.pool.connect()?)?
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
        Cacher::get(CacheId::Session, token, &mut self.pool.connect()?).map_err(Error::new)
    }

    async fn cache_session(&self, token: &str, session: &Session) -> Result<(), Error> {
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
