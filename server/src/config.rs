pub mod cache;
pub mod constants;
pub mod cors;

use crate::api::router;
use actix_web::web::ServiceConfig;
use alx_core::clients::{
    db::{postgres::Postgres, redis::Redis},
    email::Email,
};
use std::sync::Arc;
use tracing::info;

pub(super) fn init(cfg: &mut ServiceConfig) {
    let pg = Arc::new(Postgres::new());
    info!("Postgres pool initialized");

    let rd = Arc::new(Redis::new());
    info!("Redis pool initialized");

    let email = Arc::new(Email::new());
    info!("Email client initialized");

    router::auth::setup::routes(pg.clone(), rd.clone(), email, cfg);
    router::users::setup::routes(pg, rd, cfg);
    router::health::route(cfg);
    router::resources::setup::routes(cfg);
}
