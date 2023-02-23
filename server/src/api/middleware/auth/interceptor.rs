use super::adapter::{Cache, Repository};
use super::contract::{AuthGuardContract, CacheContract, RepoContract};
use super::domain::AuthenticationGuard;
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
use storage::models::role::Role;
use tracing::{debug, info};

#[derive(Debug, Clone)]
pub(crate) struct AuthGuard<Repo, Cache> {
    guard: Rc<AuthenticationGuard<Repo, Cache>>,
}

impl AuthGuard<Repository<PgSessionAdapter>, Cache> {
    pub fn new(pg: Arc<Postgres>, rd: Arc<Redis>, role: Role) -> Self {
        Self {
            guard: Rc::new(AuthenticationGuard::new(pg, rd, role)),
        }
    }
}

impl<Serv, Repo, Cache> Transform<Serv, ServiceRequest> for AuthGuard<Repo, Cache>
where
    Serv: Service<ServiceRequest, Response = ServiceResponse, Error = actix_web::Error> + 'static,
    Serv::Future: 'static,
    Repo: RepoContract + Send + Sync + 'static,
    Cache: CacheContract + Send + Sync + 'static,
{
    type Response = ServiceResponse;
    type Error = actix_web::Error;
    type InitError = ();
    type Transform = AuthMiddleware<Serv, Repo, Cache>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: Serv) -> Self::Future {
        ready(Ok(AuthMiddleware {
            service: Rc::new(service),
            guard: self.guard.clone(),
        }))
    }
}

pub(crate) struct AuthMiddleware<Serv, Repo, Cache> {
    guard: Rc<AuthenticationGuard<Repo, Cache>>,
    service: Rc<Serv>,
}

impl<Serv, Repo, Cache> Service<ServiceRequest> for AuthMiddleware<Serv, Repo, Cache>
where
    Serv: Service<ServiceRequest, Response = ServiceResponse, Error = actix_web::Error> + 'static,
    Serv::Future: 'static,
    Repo: RepoContract + Send + Sync + 'static,
    Cache: CacheContract + Send + Sync + 'static,
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
            let error_response = /* Error response for brevity */
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

            if !guard.check_valid_role(&user_sess.role) {
                return Ok(error_response(
                    Error::new(AuthenticationError::InsufficientRights),
                    req,
                ));
            }

            // Append the session to the request and call the next middleware
            req.extensions_mut().insert(user_sess);

            let res = service.call(req).await?;

            Ok(res)
        }
        .boxed_local()
    }
}
