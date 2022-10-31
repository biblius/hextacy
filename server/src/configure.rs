use crate::api::router;
use actix_web::web::ServiceConfig;
use infrastructure::clients::{
    email,
    store::{postgres::Postgres, redis::Redis},
};
use std::sync::Arc;
use tracing::info;

pub(super) fn configure(cfg: &mut ServiceConfig) {
    let pg = Arc::new(Postgres::new());
    info!("Postgres pool initialized");

    let rd = Arc::new(Redis::new());
    info!("Redis pool initialized");

    let email_client = Arc::new(email::build_client());
    info!("Email client initialized");

    router::auth::setup::routes(pg.clone(), rd.clone(), email_client.clone(), cfg);
    router::users::setup::routes(pg, rd, cfg);
}
