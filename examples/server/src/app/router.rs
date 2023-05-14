pub mod health;
pub mod resources;

use crate::config::AppState;
use crate::db::models::role::Role;
use actix_web::web;
use hextacy::route;
use hextacy::web::Configure;

use super::setup::auth_middleware::AuthenticationMiddleware;

pub fn route(state: &AppState, cfg: &mut web::ServiceConfig) {
    let auth_guard =
        AuthenticationMiddleware::new(state.pg_sea.clone(), state.redis.clone(), Role::User);

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
    use super::setup::auth_service::*;
    use crate::app::core::auth::native::handler::*;
    AuthenticationService::configure(state, cfg);
    route!(
        AuthenticationService, cfg,
        post => "/auth/login" => login;
        post => "/auth/register" =>  start_registration;
        get => "/auth/verify-registration-token" => verify_registration_token;
        post => "/auth/resend-registration-token" => resend_registration_token;
        get => "/auth/set-otp" => set_otp_secret;
        post => "/auth/verify-otp" => verify_otp;
        post => "/auth/change-password" => | auth_guard => change_password;
        post => "/auth/forgot-password" => forgot_password;
        post => "/auth/verify-forgot-password" => verify_forgot_password;
        get => "/auth/reset-password" => reset_password;
        post => "/auth/logout" => | auth_guard => logout;
    );
}

fn oauth_service(
    state: &AppState,
    cfg: &mut web::ServiceConfig,
    auth_guard: AuthenticationMiddleware,
) {
    use super::setup::oauth_service::*;
    use crate::app::core::auth::o_auth::handler::*;
    OAuthService::configure(state, cfg);
    route!(
        OAuthService, cfg,
        post => "/auth/oauth/{provider}/login" => login;
        post => "/auth/oauth/{provider}/scope" => | auth_guard => request_scopes;
    );
}

fn user_service(
    state: &AppState,
    cfg: &mut web::ServiceConfig,
    auth_guard: AuthenticationMiddleware,
) {
    use super::setup::user_service::*;
    use crate::app::core::users::handler::*;
    UserService::configure(state, cfg);
    route!(
        UserService, cfg,
        get => "/users" => | auth_guard => get_paginated
    );
}
