use super::adapter::{Cache, Repository};
use super::contract::{AuthGuardContract, CacheContract, RepositoryContract};
use crate::config::constants::COOKIE_S_ID;
use crate::db::adapters::postgres::session::PgSessionAdapter;
use crate::db::models::role::Role;
use crate::db::models::session::Session;
use crate::error::{AuthenticationError, Error};
use actix_web::{cookie::Cookie, dev::ServiceRequest};
use async_trait::async_trait;
use hextacy::clients::cache::redis::Redis;
use hextacy::clients::db::postgres::Postgres;
use std::sync::Arc;
use tracing::{debug, trace, warn};

#[derive(Debug, Clone)]
pub(super) struct AuthenticationGuard<R, C> {
    pub repository: R,
    pub cache: C,
    pub auth_level: Role,
}

impl AuthenticationGuard<Repository<PgSessionAdapter>, Cache> {
    pub fn new(pg: Arc<Postgres>, rd: Arc<Redis>, role: Role) -> Self {
        Self {
            cache: Cache { client: rd },
            repository: Repository {
                client: pg,
                _session: PgSessionAdapter,
            },
            auth_level: role,
        }
    }
}

#[async_trait(?Send)]
impl<R, C> AuthGuardContract for AuthenticationGuard<R, C>
where
    R: RepositoryContract,
    C: CacheContract,
{
    /// Attempts to get a session from the cache. If it doesn't exist, checks the database for an unexpired session.
    /// Then if the session is found and permanent, caches it. If it's not permanent, refreshes it for 30 minutes.
    /// If it can't find a session returns an `Unauthenticated` error.
    async fn get_valid_session(&self, session_id: &str, csrf: &str) -> Result<Session, Error> {
        // Check cache
        match self.cache.get_session_by_id(session_id) {
            Ok(session) => {
                if session.csrf != csrf {
                    return Err(Error::new(AuthenticationError::InvalidCsrfHeader));
                }
                trace!(
                    "Found cached session: {:?}, is permanent: {}",
                    session,
                    session.is_permanent()
                );
                if !session.is_permanent() {
                    self.cache.refresh_session(session_id)?;
                }
                Ok(session)
            }
            Err(e) => {
                trace!("{e}");
                // Check DB
                if let Ok(session) = self.extract_user_session(session_id, csrf).await {
                    debug!("Found valid session with id {}", session.id);
                    // Cache
                    self.cache.cache_session(session_id, &session)?;
                    debug!("Refreshing session {}", session.id);
                    if !session.is_permanent() {
                        self.repository.refresh_session(&session.id, csrf).await?;
                    }
                    Ok(session)
                } else {
                    warn!("No valid session found");
                    Err(Error::new(AuthenticationError::Unauthenticated))
                }
            }
        }
    }

    /// Extracts the x-csrf-token header from the request
    async fn get_csrf_header<'a>(&self, req: &'a ServiceRequest) -> Result<&'a str, Error> {
        req.headers().get("x-csrf-token").map_or_else(
            || Err(AuthenticationError::InvalidCsrfHeader.into()),
            |value| value.to_str().map_err(Error::new),
        )
    }

    /// Extracts the `S_ID` cookie from the request
    async fn get_session_cookie(&self, req: &ServiceRequest) -> Result<Cookie<'_>, Error> {
        req.cookie(COOKIE_S_ID)
            .ok_or_else(|| AuthenticationError::Unauthenticated.into())
    }

    /// Returns true if the role is equal to or greater than the auth_level of this guard instance
    #[inline]
    async fn check_valid_role(&self, role: &Role) -> bool {
        role >= &self.auth_level
    }

    async fn extract_user_session(&self, id: &str, csrf: &str) -> Result<Session, Error> {
        self.repository
            .get_valid_session(id, csrf)
            .await
            .map_err(|e| e.into())
    }
}
