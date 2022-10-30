use super::contract::{CacheContract, RepositoryContract};
use super::domain::AuthenticationGuard;
use super::infratructure::{Cache, Repository};
use crate::api::middleware::auth::contract::AuthGuardContract;
use crate::error::{AuthenticationError, Error};
use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::HttpMessage;
use futures_util::future::LocalBoxFuture;
use futures_util::FutureExt;
use infrastructure::clients::store::postgres::Postgres;
use infrastructure::clients::store::redis::Redis;
use infrastructure::store::adapters::postgres::session::PgSessionAdapter;
use infrastructure::store::adapters::postgres::user::PgUserAdapter;
use infrastructure::store::repository::role::Role;
use std::future::{ready, Ready};
use std::rc::Rc;
use std::sync::Arc;
use tracing::{debug, info};

#[derive(Debug, Clone)]
pub(crate) struct AuthGuard<R, C>
where
    R: RepositoryContract,
    C: CacheContract,
{
    guard: Rc<AuthenticationGuard<R, C>>,
}

impl AuthGuard<Repository<PgSessionAdapter, PgUserAdapter>, Cache> {
    pub fn new(pg_client: Arc<Postgres>, rd_client: Arc<Redis>, role: Role) -> Self {
        Self {
            guard: Rc::new(AuthenticationGuard::new(pg_client, rd_client, role)),
        }
    }
}

impl<S, R, C> Transform<S, ServiceRequest> for AuthGuard<R, C>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = actix_web::Error> + 'static,
    S::Future: 'static,
    R: RepositoryContract + Send + Sync + 'static,
    C: CacheContract + Send + Sync + 'static,
{
    type Response = ServiceResponse;
    type Error = actix_web::Error;
    type InitError = ();
    type Transform = AuthMiddleware<S, R, C>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware {
            service: Rc::new(service),
            guard: self.guard.clone(),
        }))
    }
}

pub(crate) struct AuthMiddleware<S, R, C>
where
    R: RepositoryContract,
    C: CacheContract,
{
    guard: Rc<AuthenticationGuard<R, C>>,
    service: Rc<S>,
}

impl<S, R, C> Service<ServiceRequest> for AuthMiddleware<S, R, C>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = actix_web::Error> + 'static,
    S::Future: 'static,
    R: RepositoryContract + Send + Sync + 'static,
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
            let user_sess = guard.get_valid_session(session_id.value(), csrf).await?;
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
