use super::contract::{AuthGuardContract, CacheContract, RepositoryContract};
use crate::error::{AuthenticationError, Error};
use crate::services::cache::{Cache as CacheService, CacheId};
use actix_web::{cookie::Cookie, dev::ServiceRequest};
use async_trait::async_trait;
use chrono::Utc;
use infrastructure::clients::store::postgres::Postgres;
use infrastructure::clients::store::redis::Commands;
use infrastructure::store::adapters::postgres::session::PgSessionAdapter;
use infrastructure::store::adapters::postgres::user::PgUserAdapter;
use infrastructure::store::adapters::AdapterError;
use infrastructure::store::models::user_session::UserSession;
use infrastructure::store::repository::user::UserRepository;
use infrastructure::{
    clients::store::redis::Redis,
    config::constants::SESSION_CACHE_DURATION_SECONDS,
    store::adapters::postgres::PgAdapterError,
    store::repository::{
        role::Role,
        session::{Session, SessionRepository},
    },
    web::http::cookie::S_ID,
};
use std::sync::Arc;
use tracing::{debug, trace, warn};

#[derive(Debug, Clone)]
pub(super) struct AuthenticationGuard<R: RepositoryContract, C: CacheContract> {
    pub repository: R,
    pub cache: C,
    pub auth_level: Role,
}

impl AuthenticationGuard<Repository<PgSessionAdapter, PgUserAdapter>, Cache> {
    pub fn new(pg_client: Arc<Postgres>, rd_client: Arc<Redis>, role: Role) -> Self {
        Self {
            repository: Repository {
                session_repo: PgSessionAdapter {
                    client: pg_client.clone(),
                },
                user_repo: PgUserAdapter {
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
    async fn get_valid_session(&self, session_id: &str, csrf: &str) -> Result<UserSession, Error> {
        // Check cache
        match self.cache.get_session_by_id(session_id).await {
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
                    self.cache.refresh_session(session_id).await?;
                }
                Ok(session)
            }
            Err(e) => {
                trace!("{e}");
                // Check DB
                if let Ok(session) = self
                    .repository
                    .get_valid_user_session(session_id, csrf)
                    .await
                {
                    debug!("Found valid session with id {}", session.id);
                    // Cache
                    self.cache.cache_session(session_id, &session).await?;
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
pub struct Repository<SR: SessionRepository, UR: UserRepository> {
    pub session_repo: SR,
    pub user_repo: UR,
}

#[async_trait]
impl<SR, UR> RepositoryContract for Repository<SR, UR>
where
    SR: SessionRepository<Error = PgAdapterError> + Send + Sync,
    UR: UserRepository<Error = PgAdapterError> + Send + Sync,
{
    /// Attempts to find an unexpired session with its corresponding CSRF
    async fn get_valid_user_session(&self, id: &str, csrf: &str) -> Result<UserSession, Error> {
        let session = self
            .session_repo
            .get_valid_by_id(id, csrf)
            .await
            .map_err(|e| AdapterError::Postgres(e))?;
        let user = self
            .user_repo
            .get_by_id(&session.user_id)
            .await
            .map_err(|e| AdapterError::Postgres(e))?;
        Ok(UserSession::new(user, session))
    }

    /// Extends session `expires_at` for 30 minutes
    async fn refresh_session(&self, id: &str, csrf: &str) -> Result<Session, Error> {
        self.session_repo
            .refresh(id, csrf)
            .await
            .map_err(|e| AdapterError::Postgres(e).into())
    }
}

#[derive(Debug, Clone)]
pub struct Cache {
    pub client: Arc<Redis>,
}

#[async_trait]
impl CacheContract for Cache {
    async fn get_session_by_id(&self, id: &str) -> Result<UserSession, Error> {
        CacheService::get(CacheId::Session, id, &mut self.client.connect()?).map_err(Error::new)
    }

    async fn cache_session(&self, id: &str, session: &UserSession) -> Result<(), Error> {
        CacheService::set(
            CacheId::Session,
            id,
            session,
            Some(SESSION_CACHE_DURATION_SECONDS),
            &mut self.client.connect()?,
        )
        .map_err(Error::new)
    }

    async fn refresh_session(&self, session_id: &str) -> Result<(), Error> {
        let mut conn = self.client.connect()?;
        conn.expire_at(
            session_id,
            ((Utc::now().timestamp() + SESSION_CACHE_DURATION_SECONDS as i64) % i64::MAX) as usize,
        )?;
        Ok(())
    }
}
