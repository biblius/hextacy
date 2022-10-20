use super::service::AuthenticationGuard;
use crate::error::{AuthenticationError, Error};
use crate::models::role::Role;
use crate::models::session::Session;
use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::HttpMessage;
use futures_util::future::LocalBoxFuture;
use futures_util::FutureExt;
use infrastructure::storage::postgres::Pg;
use infrastructure::storage::redis::Rd;
use std::future::{ready, Ready};
use std::rc::Rc;
use std::sync::Arc;
use tracing::{debug, info};

#[derive(Debug, Clone)]
pub struct Auth {
    guard: Rc<AuthenticationGuard>,
}

impl Auth {
    pub fn new(pg_pool: Arc<Pg>, rd_pool: Arc<Rd>, auth_level: Role) -> Self {
        Self {
            guard: Rc::new(AuthenticationGuard::new(pg_pool, rd_pool, auth_level)),
        }
    }
}

impl<S> Transform<S, ServiceRequest> for Auth
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = actix_web::Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = actix_web::Error;
    type InitError = ();
    type Transform = AuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware {
            service: Rc::new(service),
            guard: self.guard.clone(),
        }))
    }
}

pub struct AuthMiddleware<S> {
    guard: Rc<AuthenticationGuard>,
    service: Rc<S>,
}

impl<S> Service<ServiceRequest> for AuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = actix_web::Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        info!("Auth guard: Validating session");

        let guard = self.guard.clone();
        let service = self.service.clone();

        async move {
            // Prepare an error response for brevity
            let error_response =
                |e, r: ServiceRequest| ServiceResponse::from_err(e, r.request().to_owned());

            // Get the csrf header
            let csrf = match AuthenticationGuard::get_csrf_header(&req).await {
                Ok(token) => token,
                Err(e) => return Ok(error_response(e, req)),
            };

            debug!("Found csrf header: {csrf}");

            // Get the session ID
            let session_id = match AuthenticationGuard::get_session_cookie(&req).await {
                Ok(id) => id,
                Err(e) => return Ok(error_response(e, req)),
            };

            debug!("Found session ID cookie with value {}", session_id.value());

            // Check if the session is cached and return it in the request
            match guard.get_cached_session(csrf).await {
                Ok(session) => {
                    debug!("Found cached session with id {session_id}");

                    if !guard.check_valid_role(&session.user_role) {
                        return Ok(error_response(
                            Error::new(AuthenticationError::InsufficientRights),
                            req,
                        ));
                    }

                    req.extensions_mut().insert::<Session>(session);

                    let res = service.call(req).await?;

                    Ok(res)
                }
                Err(_) => {
                    debug!("Cached session not found, searching in PG");

                    // Otherwise check for a valid session in the db
                    let session = match guard.get_valid_session(session_id.value(), csrf).await {
                        Ok(session) => session,
                        Err(e) => return Ok(error_response(e, req)),
                    };

                    debug!("Found valid session with id {}, caching", session.id);

                    if let Err(e) = guard.refresh_and_cache(csrf, &session).await {
                        return Ok(error_response(e, req));
                    }

                    if !guard.check_valid_role(&session.user_role) {
                        return Ok(error_response(
                            Error::new(AuthenticationError::InsufficientRights),
                            req,
                        ));
                    }

                    req.extensions_mut().insert::<Session>(session);

                    let res = service.call(req).await?;

                    Ok(res)
                }
            }
        }
        .boxed_local()
    }
}
