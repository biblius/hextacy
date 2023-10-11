use crate::{
    config::state::{AppState, AuthenticationMiddleware, AuthenticationService},
    controllers::http::middleware::auth::session_check,
};
use axum::{
    middleware::{self},
    routing::{get, post},
    Router,
};

pub async fn router(state: &AppState) -> Router {
    let auth_middleware = AuthenticationMiddleware::init(state);
    let auth_service = AuthenticationService::init(state).await;

    let resource_router = resource_router();
    let auth_router = auth_router(auth_service, auth_middleware).await;

    let router = Router::new();

    router.merge(auth_router).merge(resource_router)
}

fn resource_router() -> Router {
    use crate::controllers::http::resources::*;
    let router = Router::new();
    router.route("/favicon.ico", get(favicon::favicon))
}

async fn auth_router(
    service: AuthenticationService,
    auth_mw: AuthenticationMiddleware,
) -> Router<()> {
    use crate::controllers::http::auth::*;

    let router = Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route(
            "/logout",
            post(logout).layer(middleware::from_fn_with_state(auth_mw, session_check)),
        );

    Router::new().nest("/auth", router).with_state(service)
}
