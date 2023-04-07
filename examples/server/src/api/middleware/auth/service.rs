use super::adapter::{Cache, Repository};
use super::api::{AuthGuardApi, CacheApi, RepositoryApi};
use crate::config::constants::COOKIE_S_ID;
use crate::db::models::role::Role;
use crate::db::models::session::Session;
use crate::db::repository::session::SessionRepository;
use crate::error::{AuthenticationError, Error};
use actix_web::{cookie::Cookie, dev::ServiceRequest};
use async_trait::async_trait;
use hextacy::drivers::cache::redis::Redis;
use hextacy::drivers::db::{DBConnect, Driver};
use std::marker::PhantomData;
use std::sync::Arc;
use tracing::{debug, trace, warn};

#[derive(Debug, Clone)]
pub(super) struct AuthenticationGuard<R, C> {
    pub repository: R,
    pub cache: C,
    pub auth_level: Role,
}

impl<D, C, Session> AuthenticationGuard<Repository<D, C, Session>, Cache>
where
    Session: SessionRepository<C>,
    D: DBConnect<Connection = C>,
{
    pub fn new(pg: Arc<D>, rd: Arc<Redis>, role: Role) -> Self {
        Self {
            cache: Cache { driver: rd },
            repository: Repository {
                driver: Driver::new(pg),
                _session: PhantomData,
            },
            auth_level: role,
        }
    }
}

#[async_trait]
impl<R, C> AuthGuardApi for AuthenticationGuard<R, C>
where
    R: RepositoryApi + Send + Sync,
    C: CacheApi + Send + Sync,
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
    fn get_csrf_header<'a>(&self, req: &'a ServiceRequest) -> Result<&'a str, Error> {
        req.headers().get("x-csrf-token").map_or_else(
            || Err(AuthenticationError::InvalidCsrfHeader.into()),
            |value| value.to_str().map_err(Error::new),
        )
    }

    /// Extracts the `S_ID` cookie from the request
    fn get_session_cookie(&self, req: &ServiceRequest) -> Result<Cookie<'_>, Error> {
        req.cookie(COOKIE_S_ID)
            .ok_or_else(|| AuthenticationError::Unauthenticated.into())
    }

    /// Returns true if the role is equal to or greater than the auth_level of this guard instance
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
