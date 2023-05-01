/// Used for ergonomic routing.
///
/// The syntax is as follows:
///
/// 1) Specifies the handler's service bounds. This depends on how you instantiate the service beforehand and
///    must match the instance's bounds.
///
/// 2) Actix's configuration struct for setting up routes.
///
/// 3) The HTTP method, followed by the route, followed by the handler that will handle
///    the request. Optionally, a pipe operator can be added to configure middleware
///    for the route. The middleware must be instantiated beforehand and must be cloneable.
///    This pattern is repeatable.
///
/// ```ignore
/// route!(
///    Service<Bound1, ..., BoundN>, // 1
///    cfg, // 2
///    post => "/route" => handler_function | optional_middleware, // 3
/// )
/// ```
#[macro_export]
macro_rules! route {
    (
        $service:path,
        $cfg:ident,
        $(
            $method:ident => $route:literal => $function:ident
            $(|)?
            $(
                $mw:ident
            ),*
        );* $(;)?
    ) => {
        $(
            $cfg.service(
                actix_web::web::resource($route)
                .route(actix_web::web::$method().to(handler::$function::<$service>))
                $(.wrap($mw.clone()))*
            )
        );*
    };
}

#[macro_export]
macro_rules! transform {
    (
        $from:ident => $into:ident,
        $(
            $generic:ident : $bound:ident
        ),*
     ) => {
        impl<S, $($generic),*> actix_web::dev::Transform<S, actix_web::dev::ServiceRequest> for $from<$($generic),*>
        where
            S: actix_web::dev::Service<
                actix_web::dev::ServiceRequest,
                Response = actix_web::dev::ServiceResponse,
                Error = actix_web::Error> + 'static,
            S::Future: 'static,
            $($generic: $bound + Send + Sync + 'static),*
        {
            type Response = actix_web::dev::ServiceResponse;
            type Error = actix_web::Error;
            type InitError = ();
            type Transform = $into<S, $($generic),*>;
            type Future = Ready<Result<Self::Transform, Self::InitError>>;

            fn new_transform(&self, service: S) -> Self::Future {
                ready(Ok(AuthMiddleware {
                    inner: self.inner.clone(),
                    service: Rc::new(service),
                }))
            }
        }
    };
}

#[macro_export]
macro_rules! call {
    (
        $mw:ident,
        $(
            $generic:ident : $bound:ident
        ),*
        $(,)?
        ;
        $b:item
    ) => {
        impl<S, $($generic),*> actix_web::dev::Service<actix_web::dev::ServiceRequest> for $mw<S, $($generic),*>
        where
            S: actix_web::dev::Service<
                actix_web::dev::ServiceRequest,
                Response = actix_web::dev::ServiceResponse,
                Error = actix_web::Error> + 'static,
            S::Future: 'static,
            $($generic: $bound + Send + Sync + 'static),*
        {
            type Response = actix_web::dev::ServiceResponse;
            type Error = actix_web::Error;
            type Future = futures_util::future::LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

            actix_web::dev::forward_ready!(service);

            $b
        }
    }
}
