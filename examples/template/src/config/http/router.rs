use super::middleware::AuthenticationMiddleware;
use crate::{
    cache::adapters::RedisAdapter,
    config::state::AppState,
    controllers::http::middleware::auth::session_check,
    db::adapters::{session::SessionAdapter, user::UserAdapter},
};
use axum::{
    middleware::{self},
    routing::{get, post},
    Router,
};

pub fn router(state: &AppState) -> Router {
    let auth_middleware = AuthenticationMiddleware::init(state);

    let resource_router = resource_router();
    let auth_router = auth_router(state, auth_middleware);

    let router = Router::new();

    router.merge(auth_router).merge(resource_router)
}

fn resource_router() -> Router {
    use crate::controllers::http::resources::*;
    let router = Router::new();
    router.route("/favicon.ico", get(favicon::favicon))
}

use crate::config::AuthenticationService;
fn auth_router(state: &AppState, auth_mw: AuthenticationMiddleware) -> Router<()> {
    use crate::controllers::http::auth::handler::*;

    let service = AuthenticationService::new(
        state.repository.clone(),
        state.cache.clone(),
        UserAdapter,
        SessionAdapter,
        RedisAdapter,
    );

    let router = Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route(
            "/logout",
            post(logout).layer(middleware::from_fn_with_state(auth_mw, session_check)),
        );

    Router::new().nest("/auth", router).with_state(service)

    /*     scope! {
        AuthenticationService, cfg, "/auth",
        post => "/register" => register;
        post => "/login" => login;
        post => "/logout" => | auth_mw => logout;
    }; */
}
