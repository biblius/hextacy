pub mod constants;
pub mod cors;

use crate::{app::router, services::email::Email};
use actix_web::web::ServiceConfig;
use hextacy::{
    adapters::{
        cache::redis::Redis,
        db::{
            mongo::Mongo,
            postgres::{diesel::PostgresDiesel, seaorm::PostgresSea},
        },
    },
    Configuration,
};
use std::sync::Arc;
use tracing::info;

#[derive(Debug, Clone, Configuration)]
pub struct AppState {
    #[env(
        "PG_HOST",
        "PG_PORT" as u16,
        "PG_USER",
        "PG_PASSWORD",
        "PG_DATABASE",
        "PG_POOL_SIZE" as Option<u32>
    )]
    pub pg_diesel: Arc<PostgresDiesel>,

    /*     #[env(
        "PG_HOST",
        "PG_PORT",
        "PG_USER",
        "PG_PASSWORD",
        "PG_DATABASE",
        "PG_POOL_SIZE"
    )] */
    pub pg_sea: Arc<PostgresSea>,
    // #[raw("localhost", 6379, None, None, 0, 8)]
    #[env(
        "RD_HOST",
        "RD_PORT" as u16,
        "RD_USER" as Option,
        "RD_PASSWORD" as Option,
        "RD_DATABASE" as i64,
        "RD_POOL_SIZE" as usize
    )]
    pub redis: Arc<Redis>,
    pub smtp: Arc<Email>,
    //    #[env("MONGO_HOST", "MONGO_PORT")]
    pub mongo: std::sync::Arc<Mongo>,
}

impl AppState {
    pub async fn init() -> Self {
        let pg_diesel = init_diesel_pg();
        // info!("PostgresDiesel pool initialized");

        let pg_sea = init_sea_pg().await;
        // info!("PostgresSea pool initialized");

        let mongo = init_mongo();
        // info!("Mongo pool initialized");

        let redis = init_rd();
        // info!("Redis pool initialized");

        let smtp = init_email();
        // info!("Email client initialized");

        Self {
            pg_diesel,
            pg_sea,
            redis,
            smtp,
            mongo,
        }
    }
}

fn init_email() -> Arc<Email> {
    let params =
        crate::env::get_multiple(&["SMTP_HOST", "SMTP_PORT", "SMTP_USERNAME", "SMTP_PASSWORD"]);

    let password = &params["SMTP_PASSWORD"];
    let username = &params["SMTP_USERNAME"];
    let port = &params["SMTP_PORT"];
    let host = &params["SMTP_HOST"];

    Arc::new(Email::new(
        host,
        port.parse::<u16>().expect("Invalid SMTP port"),
        username.to_string(),
        password.to_string(),
    ))
}

pub(super) fn init(cfg: &mut ServiceConfig, state: &AppState) {
    router::route(state, cfg);
}

async fn init_sea_pg() -> Arc<PostgresSea> {
    let params = hextacy::env::get_multiple(&[
        "PG_USER",
        "PG_PASSWORD",
        "PG_HOST",
        "PG_PORT",
        "PG_DATABASE",
        "PG_POOL_SIZE",
    ]);

    let pool_size = &params["PG_POOL_SIZE"];
    let db = &params["PG_DATABASE"];
    let port = &params["PG_PORT"];
    let host = &params["PG_HOST"];
    let pw = &params["PG_PASSWORD"];
    let user = &params["PG_USER"];

    Arc::new(
        PostgresSea::new(
            host,
            port.parse().expect("Invalid PG_PORT"),
            user,
            pw,
            db,
            pool_size.parse().expect("Invalid PG_POOL_SIZE"),
        )
        .await,
    )
}

fn init_diesel_pg() -> Arc<PostgresDiesel> {
    let params = hextacy::env::get_multiple(&[
        "PG_USER",
        "PG_PASSWORD",
        "PG_HOST",
        "PG_PORT",
        "PG_DATABASE",
        "PG_POOL_SIZE",
    ]);

    let pool_size = &params["PG_POOL_SIZE"];
    let db = &params["PG_DATABASE"];
    let port = &params["PG_PORT"];
    let host = &params["PG_HOST"];
    let pw = &params["PG_PASSWORD"];
    let user = &params["PG_USER"];

    Arc::new(PostgresDiesel::new(
        host,
        port.parse().expect("Invalid PG_PORT"),
        user,
        pw,
        db,
        Some(pool_size.parse().expect("Invalid PG_POOL_SIZE")),
    ))
}

fn init_mongo() -> Arc<Mongo> {
    let params = hextacy::env::get_multiple(&[
        "MONGO_USER",
        "MONGO_PASSWORD",
        "MONGO_HOST",
        "MONGO_PORT",
        "MONGO_DATABASE",
    ]);

    let db = &params["MONGO_DATABASE"];
    let port = &params["MONGO_PORT"];
    let host = &params["MONGO_HOST"];
    let pw = &params["MONGO_PASSWORD"];
    let user = &params["MONGO_USER"];

    Arc::new(Mongo::new(
        host,
        port.parse().expect("Invalid MONGO_PORT"),
        user,
        pw,
        db,
    ))
}

fn init_rd() -> Arc<Redis> {
    let params = hextacy::env::get_multiple(&["RD_HOST", "RD_PORT", "RD_DATABASE", "RD_POOL_SIZE"]);
    let pool_size = &params["RD_POOL_SIZE"];
    let db = &params["RD_DATABASE"];
    let port = &params["RD_PORT"];
    let host = &params["RD_HOST"];

    Arc::new(Redis::new(
        host,
        port.parse().expect("Invalid RD_PORT"),
        None,
        None,
        db.parse().expect("Invalid RD_DATABASE"),
        pool_size.parse().expect("Invalid RD_POOL_SIZE"),
    ))
}
