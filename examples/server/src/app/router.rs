pub mod health;
pub mod resources;

use super::setup::auth_middleware::AuthenticationMiddleware;
use crate::config::AppState;
use crate::services::oauth::OAuthProviders;
use actix_web::{body::MessageBody, web, Responder};
use hextacy::{
    route, scope,
    web::{http::Response, Configure},
};

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
