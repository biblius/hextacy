pub mod cache;
pub mod constants;
pub mod cors;

use crate::api::router;
use actix_web::web::ServiceConfig;
use hextacy::drivers::{
    cache::redis::Redis,
    db::{
        mongo::Mongo,
        postgres::{diesel::PostgresDiesel, seaorm::PostgresSea},
    },
    email::Email,
};
use std::sync::Arc;
use tracing::info;

#[derive(Debug, Clone)]
pub struct AppState {
    pub pg_diesel: Arc<PostgresDiesel>,
    pub pg_sea: Arc<PostgresSea>,
    pub redis: Arc<Redis>,
    pub smtp: Arc<Email>,
    pub mongo: Arc<Mongo>,
}

impl AppState {
    pub async fn init() -> Self {
        let pg_diesel = init_diesel_pg();
        info!("PostgresDiesel pool initialized");

        let pg_sea = init_sea_pg().await;
        info!("PostgresSea pool initialized");

        let mongo = init_mongo();
        info!("Mongo pool initialized");

        let redis = init_rd();
        info!("Redis pool initialized");

        let smtp = Arc::new(Email::new());
        info!("Email client initialized");

        Self {
            pg_diesel,
            pg_sea,
            redis,
            smtp,
            mongo,
        }
    }
}

pub(super) fn init(cfg: &mut ServiceConfig, state: &AppState) {
    router::auth::native::setup::routes(state, cfg);

    router::auth::o_auth::setup::routes(state, cfg);
    router::users::setup::routes(state.pg_diesel.clone(), state.redis.clone(), cfg);
    router::health::route(cfg);
    router::resources::setup::routes(cfg);
}

async fn init_sea_pg() -> Arc<PostgresSea> {
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

    Arc::new(
        PostgresSea::new(
            &host,
            port.parse().expect("Invalid PG_PORT"),
            &user,
            &pw,
            &db,
            pool_size.parse().expect("Invalid PG_POOL_SIZE"),
        )
        .await,
    )
}

fn init_diesel_pg() -> Arc<PostgresDiesel> {
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

    Arc::new(PostgresDiesel::new(
        &host,
        port.parse().expect("Invalid PG_PORT"),
        &user,
        &pw,
        &db,
        pool_size.parse().expect("Invalid PG_POOL_SIZE"),
    ))
}

fn init_mongo() -> Arc<Mongo> {
    let mut params = hextacy::env::get_multiple(&[
        "MONGO_USER",
        "MONGO_PASSWORD",
        "MONGO_HOST",
        "MONGO_PORT",
        "MONGO_DATABASE",
    ]);

    let db = params.pop().expect("MONGO_DATABASE must be set");
    let port = params.pop().expect("MONGO_PORT must be set");
    let host = params.pop().expect("MONGO_HOST must be set");
    let pw = params.pop().expect("MONGO_PASSWORD must be set");
    let user = params.pop().expect("MONGO_USER must be set");

    Arc::new(Mongo::new(
        &host,
        port.parse().expect("Invalid MONGO_PORT"),
        &user,
        &pw,
        &db,
    ))
}

fn init_rd() -> Arc<Redis> {
    let mut params =
        hextacy::env::get_multiple(&["RD_HOST", "RD_PORT", "RD_DATABASE", "RD_POOL_SIZE"]);
    let pool_size = params.pop().expect("RD_POOL_SIZE must be set");
    let db = params.pop().expect("RD_DATABASE must be set");
    let port = params.pop().expect("RD_PORT must be set");
    let host = params.pop().expect("RD_HOST must be set");

    Arc::new(Redis::new(
        &host,
        port.parse().expect("Invalid RD_PORT"),
        None,
        None,
        db.parse().expect("Invalid RD_DATABASE"),
        pool_size.parse().expect("Invalid RD_POOL_SIZE"),
    ))
}
