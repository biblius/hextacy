pub mod cache;
pub mod constants;
pub mod cors;

use crate::api::router;
use actix_web::web::ServiceConfig;
use hextacy::clients::{cache::redis::Redis, db::postgres::Postgres, email::Email};
use std::sync::Arc;
use tracing::info;

pub(super) fn init(cfg: &mut ServiceConfig) {
    let pg = init_pg();
    info!("Postgres pool initialized");

    let rd = init_rd();
    info!("Redis pool initialized");

    let email = Arc::new(Email::new());
    info!("Email client initialized");

    router::auth::native::setup::routes(pg.clone(), rd.clone(), email, cfg);
    router::auth::o_auth::setup::google::routes(pg.clone(), rd.clone(), cfg);
    router::auth::o_auth::setup::github::routes(pg.clone(), rd.clone(), cfg);
    router::users::setup::routes(pg, rd, cfg);
    router::health::route(cfg);
    router::resources::setup::routes(cfg);
}

fn init_pg() -> Arc<Postgres> {
    let mut params = hextacy::env::get_multiple(&[
        "PG_USER",
        "PG_PASSWORD",
        "PG_HOST",
        "PG_PORT",
        "PG_DATABASE",
        "PG_POOL_SIZE",
    ]);
    let pool_size = params.pop().expect("PG_POOL_SIZE must be set");
    let db = params.pop().expect("PG_DATABASE must be set");
    let port = params.pop().expect("PG_PORT must be set");
    let host = params.pop().expect("PG_HOST must be set");
    let pw = params.pop().expect("PG_PASSWORD must be set");
    let user = params.pop().expect("PG_USER must be set");

    Arc::new(Postgres::new(
        &host,
        port.parse().expect("Invalid PG_PORT"),
        &user,
        &pw,
        &db,
        pool_size.parse().expect("Invalid PG_POOL_SIZE"),
    ))
}

fn init_rd() -> Arc<Redis> {
    let mut params = hextacy::env::get_multiple(&[
        "RD_USER",
        "RD_PASSWORD",
        "RD_HOST",
        "RD_PORT",
        "RD_DATABASE",
        "RD_POOL_SIZE",
    ]);
    let pool_size = params.pop().expect("RD_POOL_SIZE must be set");
    let db = params.pop().expect("RD_DATABASE must be set");
    let port = params.pop().expect("RD_PORT must be set");
    let host = params.pop().expect("RD_HOST must be set");
    let pw = params.pop().expect("RD_PASSWORD must be set");
    let user = params.pop().expect("RD_USER must be set");

    Arc::new(Redis::new(
        &host,
        port.parse().expect("Invalid RD_PORT"),
        &user,
        &pw,
        db.parse().expect("Invalid RD_DATABASE"),
        pool_size.parse().expect("Invalid RD_POOL_SIZE"),
    ))
}
