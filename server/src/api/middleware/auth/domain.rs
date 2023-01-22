use super::contract::{AuthGuardContract, CacheContract};
use super::infratructure::Cache;
use crate::config::constants::COOKIE_S_ID;
use crate::error::{AuthenticationError, Error};
use actix_web::{cookie::Cookie, dev::ServiceRequest};
use infrastructure::clients::postgres::Postgres;
use infrastructure::clients::redis::Redis;
use std::sync::Arc;
use storage::adapters::postgres::session::PgSessionAdapter;
use storage::adapters::postgres::user::PgUserAdapter;
use storage::models::role::Role;
use storage::models::session::UserSession;
use storage::repository::session::SessionRepository;
use storage::repository::user::UserRepository;
use tracing::{debug, trace, warn};

#[derive(Debug, Clone)]
pub(super) struct AuthenticationGuard<UR, SR, C>
where
    UR: UserRepository,
    SR: SessionRepository,
    C: CacheContract,
{
    pub user_repo: UR,
    pub session_repo: SR,
    pub cache: C,
    pub auth_level: Role,
}

impl AuthenticationGuard<PgUserAdapter, PgSessionAdapter, Cache> {
    pub fn new(pg_client: Arc<Postgres>, rd_client: Arc<Redis>, role: Role) -> Self {
        Self {
            session_repo: PgSessionAdapter {
                client: pg_client.clone(),
            },
            user_repo: PgUserAdapter { client: pg_client },
            cache: Cache { client: rd_client },
            auth_level: role,
        }
    }
}

impl<UR, SR, C> AuthGuardContract for AuthenticationGuard<UR, SR, C>
where
    UR: UserRepository + Send + Sync,
    SR: SessionRepository + Send + Sync,
    C: CacheContract + Send + Sync,
{
    /// Attempts to get a session from the cache. If it doesn't exist, checks the database for an unexpired session.
    /// Then if the session is found and permanent, caches it. If it's not permanent, refreshes it for 30 minutes.
    /// If it can't find a session returns an `Unauthenticated` error.
    fn get_valid_session(&self, session_id: &str, csrf: &str) -> Result<UserSession, Error> {
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
                if let Ok(session) = self.extract_user_session(session_id, csrf) {
                    debug!("Found valid session with id {}", session.id);
                    // Cache
                    self.cache.cache_session(session_id, &session)?;
                    debug!("Refreshing session {}", session.id);
                    if !session.is_permanent() {
                        self.session_repo.refresh(&session.id, csrf)?;
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
    #[inline]
    fn check_valid_role(&self, role: &Role) -> bool {
        role >= &self.auth_level
    }

    fn extract_user_session(&self, id: &str, csrf: &str) -> Result<UserSession, Error> {
        let session = self.session_repo.get_valid_by_id(id, csrf)?;
        let user = self.user_repo.get_by_id(&session.user_id)?;
        Ok(UserSession::new(user, session))
    }
}
