use super::adapter::{AuthMwCacheContract, AuthMwRepoContract};
use crate::config::constants::COOKIE_S_ID;
use crate::db::models::role::Role;
use crate::db::models::session::Session;
use crate::error::{AuthenticationError, Error};
use actix_web::cookie::Cookie;
use actix_web::dev::ServiceRequest;
use actix_web::HttpMessage;
use futures_util::FutureExt;
use hextacy::{call, transform};
use std::rc::Rc;
use tracing::{debug, info};
use tracing::{trace, warn};

#[derive(Debug, Clone)]
pub(super) struct AuthenticationGuardInner<R, C> {
    pub repository: R,
    pub cache: C,
    pub auth_level: Role,
}

#[derive(Debug, Clone)]
pub struct AuthenticationGuard<Repo, Cache> {
    pub(super) inner: Rc<AuthenticationGuardInner<Repo, Cache>>,
}

#[derive(Debug, Clone)]
pub struct AuthMiddleware<S, Repo, Cache> {
    inner: Rc<AuthenticationGuardInner<Repo, Cache>>,
    service: Rc<S>,
}

transform! {
    AuthenticationGuard => AuthMiddleware,
    R: AuthMwRepoContract,
    C: AuthMwCacheContract
}

call! {
    AuthMiddleware,
    R: AuthMwRepoContract,
    C: AuthMwCacheContract;

    fn call(&self, req: ServiceRequest) -> Self::Future {
        info!("Auth guard: Validating session");

        let guard = self.inner.clone();
        let service = self.service.clone();

        async move {
            // Get the csrf header
            let csrf = match guard.get_csrf_header(&req) {
                Ok(token) => token,
                Err(e) => return Err(e.into()),
            };

            debug!("Found csrf header: {csrf}");

            // Get the session ID
            let session_id = match guard.get_session_cookie(&req) {
                Ok(id) => id,
                Err(e) => return Err(e.into()),
            };

            debug!("Found session ID cookie {session_id}");

            let user_sess = guard.get_valid_session(session_id.value(), csrf).await?;

            if !guard.check_valid_role(&user_sess.role) {
                return Err(Error::new(AuthenticationError::InsufficientRights).into());
            }

            // Append the session to the request and call the next middleware
            req.extensions_mut().insert(user_sess);

            let res = service.call(req).await?;

            Ok(res)
        }
        .boxed_local()
    }
}

impl<R, C> AuthenticationGuardInner<R, C>
where
    R: AuthMwRepoContract + Send + Sync,
    C: AuthMwCacheContract + Send + Sync,
{
    /// Attempts to get a session from the cache. If it doesn't exist, checks the database for an unexpired session.
    /// Then if the session is found and permanent, caches it. If it's not permanent, refreshes it for 30 minutes.
    /// If it can't find a session returns an `Unauthenticated` error.
    async fn get_valid_session(&self, session_id: &str, csrf: &str) -> Result<Session, Error> {
        // Check cache
        match self.cache.get_session_by_id(session_id).await {
            Ok(session) => {
                if session.csrf != csrf {
                    return Err(Error::new(AuthenticationError::InvalidCsrfHeader));
                }

                if !session.is_permanent() {
                    self.cache.refresh_session(session_id).await?;
                }

                Ok(session)
            }
            Err(e) => {
                trace!("{e}");
                // Check DB
                if let Ok(session) = self.repository.get_valid_session(session_id, csrf).await {
                    debug!("Found valid session with id {session_id}");
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
    fn get_session_cookie<'a>(&self, req: &ServiceRequest) -> Result<Cookie<'a>, Error> {
        req.cookie(COOKIE_S_ID)
            .ok_or_else(|| AuthenticationError::Unauthenticated.into())
    }

    /// Returns true if the role is equal to or greater than the auth_level of this guard instance
    fn check_valid_role(&self, role: &Role) -> bool {
        role >= &self.auth_level
    }
}
