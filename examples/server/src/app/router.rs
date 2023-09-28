pub mod health;
pub mod resources;

use super::setup::auth_middleware::AuthenticationMiddleware;
use crate::config::AppState;
use crate::route;
use crate::scope;
use crate::services::oauth::OAuthProviders;
use actix_web::{body::MessageBody, web, Responder};
use hextacy::web::http::Response;
use hextacy::Configure;

pub fn route(state: &AppState, cfg: &mut web::ServiceConfig) {
    let auth_guard = AuthenticationMiddleware::new(state);

    health::route(cfg);
    resources::route(cfg);
    auth_service(state, cfg, auth_guard.clone());
    oauth_service(state, cfg, auth_guard.clone());
    user_service(state, cfg, auth_guard)
}

fn auth_service(
    state: &AppState,
    cfg: &mut web::ServiceConfig,
    auth_guard: AuthenticationMiddleware,
) {
    use super::controllers::auth::native::*;
    use super::setup::auth_service::*;
    AuthenticationService::configure(state, cfg);
    scope!(
        AuthenticationService, cfg, "/auth",
        post => "/login" => login;
        post => "/register" =>  start_registration;
        get => "/verify-registration-token" => verify_registration_token;
        post => "/resend-registration-token" => resend_registration_token;
        get => "/set-otp" => set_otp_secret;
        post => "/verify-otp" => verify_otp;
        post => "/change-password" => | auth_guard => change_password;
        post => "/forgot-password" => forgot_password;
        post => "/verify-forgot-password" => verify_forgot_password;
        get => "/reset-password" => reset_password;
        post => "/logout" => | auth_guard => logout;
    );
}

fn oauth_service(
    state: &AppState,
    cfg: &mut web::ServiceConfig,
    auth_guard: AuthenticationMiddleware,
) {
    use super::controllers::auth::o_auth::*;
    use super::setup::oauth_service::*;
    OAuthProviders::configure(state, cfg);
    OAuthService::configure(state, cfg);
    scope!(
        OAuthService, cfg, "/oauth/{provider}",
        post => "/login" => login;
        post => "/scope" => | auth_guard => request_scopes;
    );
}

fn user_service(
    state: &AppState,
    cfg: &mut web::ServiceConfig,
    auth_guard: AuthenticationMiddleware,
) {
    use super::controllers::users::*;
    use super::setup::user_service::*;
    UserService::configure(state, cfg);
    route!(
        UserService, cfg,
        get => "/users" => | auth_guard => get_paginated
    );
}

pub struct AppResponse<T>(pub Response<T>);

impl<T: MessageBody + 'static> Responder for AppResponse<T> {
    type Body = T;

    fn respond_to(self, req: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        let mut res = actix_web::HttpResponse::new(self.0.status());
        let (parts, body) = self.0.into_parts();
        res.headers_mut().reserve(parts.headers.len());
        for (h, v) in parts.headers {
            res.headers_mut().append(h.unwrap(), v);
        }
        res.set_body(body).respond_to(req)
    }
}

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
///    post => "/route" => [ ( | optional_middleware )+ => ] handler // 3
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
