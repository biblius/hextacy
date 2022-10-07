/* use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};
use std::task::{Context, Poll};
use tracing::info;
use tracing::log::warn;

pub struct LoggedGuard;

impl<S> Transform<S, ServiceRequest> for LoggedGuard
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = actix_web::Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = actix_web::Error;
    type InitError = ();
    type Transform = LoggedGuardMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(LoggedGuardMiddleware { service }))
    }
}

pub struct LoggedGuardMiddleware<S> {
    service: S,
}

impl<S> Service<ServiceRequest> for LoggedGuardMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = actix_web::Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        match is_authorized(&req) {
            Ok(chat_user) => {
                let fut = self.service.call(req);
                info!(
                    "{}{:?}",
                    "Authentication successful, received : ", chat_user
                );
                Box::pin(async move {
                    let res = fut.await?;
                    Ok(res)
                })
            }
            Err(e) => Box::pin(async move {
                warn!("{}{:?}", "ERROR : ", e);
                Err(ServiceResponse::new(
                    req.into_parts().0,
                    GlobalError::respond(e),
                ))
            }),
        }
    }
}
 */
