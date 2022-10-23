use super::contract::{AuthGuardContract, CacheContract, RepositoryContract};
use crate::error::{AuthenticationError, Error};
use crate::services::cache::{Cache as CacheService, CacheId};
use actix_web::{cookie::Cookie, dev::ServiceRequest};
use async_trait::async_trait;
use infrastructure::adapters::postgres::session::PgSessionAdapter;
use infrastructure::clients::postgres::Postgres;
use infrastructure::{
    adapters::postgres::PgAdapterError,
    clients::redis::Redis,
    config::constants::SESSION_CACHE_DURATION_SECONDS,
    repository::{
        role::Role,
        session::{Session, SessionRepository},
    },
    web::http::cookie::S_ID,
};
use std::sync::Arc;
use tracing::debug;

#[derive(Debug, Clone)]
pub(super) struct AuthenticationGuard<R: RepositoryContract, C: CacheContract> {
    pub repository: R,
    pub cache: C,
    pub auth_level: Role,
}

impl AuthenticationGuard<Repository<PgSessionAdapter>, Cache> {
    pub fn new(pg_client: Arc<Postgres>, rd_client: Arc<Redis>, role: Role) -> Self {
        Self {
            repository: Repository {
                session_repo: PgSessionAdapter {
                    client: pg_client.clone(),
                },
            },
            cache: Cache {
                client: rd_client.clone(),
            },
            auth_level: role,
        }
    }
}

#[async_trait]
impl<R, C> AuthGuardContract for AuthenticationGuard<R, C>
where
    R: RepositoryContract + Send + Sync,
    C: CacheContract + Send + Sync,
{
    /// Attempts to get a session from the cache. If it doesn't exist, checks the database for an unexpired session.
    /// Then if the session is found and permanent, caches it. If it's not permanent, refreshes it for 30 minutes.
    /// If it can't find a session returns an `Unauthenticated` error.
    async fn get_valid_session(&self, session_id: &str, csrf: &str) -> Result<Session, Error> {
        // Check cache
        if let Ok(session) = self.cache.get_session_by_csrf(csrf).await {
            debug!("Found cached session with {session_id}");
            Ok(session)
        } else {
            // Check DB
            if let Ok(session) = self.repository.get_valid_session(session_id, csrf).await {
                debug!("Found valid session with id {}", session.id);

                // Cache if permanent
                if session.is_permanent() {
                    debug!("Session permanent, caching {}", session.id);
                    self.cache.cache_session(csrf, &session).await?;
                    return Ok(session);
                }

                // Otherwise refresh
                debug!("Refreshing session {}", session.id);
                self.repository.refresh_session(&session.id, csrf).await?;
                Ok(session)
            } else {
                Err(Error::new(AuthenticationError::Unauthenticated))
            }
        }
    }

    /// Extracts the x-csrf-token header from the request
    fn get_csrf_header<'a>(&self, req: &'a ServiceRequest) -> Result<&'a str, Error> {
        req.headers().get("x-csrf-token").map_or_else(
            || Err(AuthenticationError::InvalidCsrfHeader.into()),
            |value| value.to_str().map_err(Error::new),
        )
    }

    /// Extracts the `S_ID` cookie from the request
    fn get_session_cookie(&self, req: &ServiceRequest) -> Result<Cookie<'_>, Error> {
        req.cookie(S_ID)
            .ok_or_else(|| AuthenticationError::Unauthenticated.into())
    }

    /// Returns true if the role is equal to or greater than the auth_level of this guard instance
    #[inline]
    fn check_valid_role(&self, role: &Role) -> bool {
        role >= &self.auth_level
    }
}

#[derive(Debug, Clone)]
pub struct Repository<SR: SessionRepository> {
    pub session_repo: SR,
}

#[async_trait]
impl<SR> RepositoryContract for Repository<SR>
where
    SR: SessionRepository<Error = PgAdapterError> + Send + Sync,
{
    /// Attempts to find an unexpired session with its corresponding CSRF
    async fn get_valid_session(&self, id: &str, csrf: &str) -> Result<Session, Error> {
        self.session_repo
            .get_valid_by_id(id, csrf)
            .await
            .map_err(Error::new)
    }

    /// Extends session `expires_at` for 30 minutes
    async fn refresh_session(&self, id: &str, csrf: &str) -> Result<Session, Error> {
        self.session_repo
            .refresh(id, csrf)
            .await
            .map_err(Error::new)
    }
}

#[derive(Debug, Clone)]
pub struct Cache {
    pub client: Arc<Redis>,
}

#[async_trait]
impl CacheContract for Cache {
    async fn get_session_by_csrf(&self, token: &str) -> Result<Session, Error> {
        CacheService::get(CacheId::Session, token, &mut self.client.connect()?).map_err(Error::new)
    }

    async fn cache_session(&self, token: &str, session: &Session) -> Result<(), Error> {
        CacheService::set(
            CacheId::Session,
            token,
            session,
            Some(SESSION_CACHE_DURATION_SECONDS),
            &mut self.client.connect()?,
        )
        .map_err(Error::new)
    }
}
