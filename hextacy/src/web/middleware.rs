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
