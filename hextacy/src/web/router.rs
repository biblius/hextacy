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
///    Service, // 1
///    cfg, // 2
///    post => "/route" => handler_function | optional_middleware => fn_ident // 3
/// )
/// ```
#[macro_export]
macro_rules! route {
    (
        $service:path,
        $cfg:ident,
        $(
            $method:ident => $route:literal => $(| $($mw:ident),* =>)? $function:ident
        );* $(;)?
    ) => {
        $(
            $cfg.service(
                actix_web::web::resource($route)
                .route(actix_web::web::$method().to($function::<$service>))
                $($(.wrap($mw.clone()))*)?
            )
        );*
    };

    (
        $cfg:ident,
        $(
            $method:ident => $route:literal => $(| $($mw:ident),* =>)? $function:ident
        );* $(;)?
    ) => {
        $(
            $cfg.service(
                actix_web::web::resource($route)
                .route(actix_web::web::$method().to($function))
                $($(.wrap($mw.clone()))*)?
            )
        );*
    };
}

#[macro_export]
macro_rules! scope {
    (
        $service:path,
        $cfg:ident,
        $scope:literal,
        $(
            $method:ident => $route:literal => $(| $($mw:ident),* =>)? $function:ident
        );* $(;)?
    ) => {
        $cfg.service(
            actix_web::web::scope($scope)
                $(
                    .service(
                        actix_web::web::resource($route)
                        .route(actix_web::web::$method().to($function::<$service>))
                        $( $( .wrap($mw.clone()) )* )?
                    )
                )*
        )
    };

    (
        $cfg:ident,
        $scope:literal,
        $(
            $method:ident => $route:literal => $(| $($mw:ident),* =>)? $function:ident
        );* $(;)?
    ) => {
        $cfg.service(
            actix_web::web::scope($scope)
                $(
                    .service(
                        actix_web::web::resource($route)
                        .route(actix_web::web::$method().to($function))
                        $( $( .wrap($mw.clone()) )* )?
                    )
                )*
        )
    };
}
