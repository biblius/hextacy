use super::contract::{AuthGuardContract, CacheContract};
use super::domain::AuthenticationGuard;
use super::infratructure::Cache;
use crate::error::{AuthenticationError, Error};
/* use ::alx_core::clients::postgres::Postgres;
use ::alx_core::clients::redis::Redis; */
use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::HttpMessage;
use alx_core::clients::db::postgres::Postgres;
use alx_core::clients::db::redis::Redis;
use futures_util::future::LocalBoxFuture;
use futures_util::FutureExt;
use std::future::{ready, Ready};
use std::rc::Rc;
use std::sync::Arc;
use storage::adapters::postgres::session::PgSessionAdapter;
use storage::adapters::postgres::user::PgUserAdapter;
use storage::models::role::Role;
use storage::repository::session::SessionRepository;
use storage::repository::user::UserRepository;
use tracing::{debug, info};

#[derive(Debug, Clone)]
pub(crate) struct AuthGuard<UR, SR, C>
where
    UR: UserRepository,
    SR: SessionRepository,
    C: CacheContract,
{
    guard: Rc<AuthenticationGuard<UR, SR, C>>,
}

impl AuthGuard<PgUserAdapter, PgSessionAdapter, Cache> {
    pub fn new(pg_client: Arc<Postgres>, rd_client: Arc<Redis>, role: Role) -> Self {
        Self {
            guard: Rc::new(AuthenticationGuard::new(pg_client, rd_client, role)),
        }
    }
}

impl<S, UR, SR, C> Transform<S, ServiceRequest> for AuthGuard<UR, SR, C>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = actix_web::Error> + 'static,
    S::Future: 'static,
    UR: UserRepository + Send + Sync + 'static,
    SR: SessionRepository + Send + Sync + 'static,
    C: CacheContract + Send + Sync + 'static,
{
    type Response = ServiceResponse;
    type Error = actix_web::Error;
    type InitError = ();
    type Transform = AuthMiddleware<S, UR, SR, C>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware {
            service: Rc::new(service),
            guard: self.guard.clone(),
        }))
    }
}

pub(crate) struct AuthMiddleware<S, UR, SR, C>
where
    UR: UserRepository,
    SR: SessionRepository,
    C: CacheContract,
{
    guard: Rc<AuthenticationGuard<UR, SR, C>>,
    service: Rc<S>,
}

impl<S, UR, SR, C> Service<ServiceRequest> for AuthMiddleware<S, UR, SR, C>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = actix_web::Error> + 'static,
    S::Future: 'static,
    UR: UserRepository + Send + Sync + 'static,
    SR: SessionRepository + Send + Sync + 'static,
    C: CacheContract + Send + Sync + 'static,
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
            let csrf = match guard.get_csrf_header(&req) {
                Ok(token) => token,
                Err(e) => return Ok(error_response(e, req)),
            };
            debug!("Found csrf header: {csrf}");
            // Get the session ID
            let session_id = match guard.get_session_cookie(&req) {
                Ok(id) => id,
                Err(e) => return Ok(error_response(e, req)),
            };
            debug!("Found session ID cookie {session_id}");
            let user_sess = guard.get_valid_session(session_id.value(), csrf)?;
            if !guard.check_valid_role(&user_sess.user_role) {
                return Ok(error_response(
                    Error::new(AuthenticationError::InsufficientRights),
                    req,
                ));
            }
            req.extensions_mut().insert(user_sess);
            let res = service.call(req).await?;

            Ok(res)
        }
        .boxed_local()
    }
}
