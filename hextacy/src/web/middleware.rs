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
            type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

            fn new_transform(&self, service: S) -> Self::Future {
                std::future::ready(Ok($into {
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
            type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>>>>;

            #[inline]
            fn poll_ready(
                &self,
                cx: &mut ::core::task::Context<'_>,
            ) -> ::core::task::Poll<Result<(), Self::Error>> {
                self.service
                    .poll_ready(cx)
                    .map_err(::core::convert::Into::into)
            }

            $b
        }
    }
}
