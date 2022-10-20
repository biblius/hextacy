use actix_web::{middleware::Logger, App, HttpServer};
use infrastructure::{
    config::{env, logger},
    email, http,
    storage::{postgres::Pg, redis::Rd},
};
use std::sync::Arc;
use tracing::info;

use api::router;
mod api;
mod error;
mod models;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env::load_from_file("./.env").unwrap();

    logger::init("debug");
    // logger::init_file("debug", "server.log");

    let pg = Arc::new(Pg::new());
    info!("Postgres pool initialized");

    let rd = Arc::new(Rd::new());
    info!("Redis pool initialized");

    let email_client = Arc::new(email::build_client());
    info!("Email client initialized");

    let (host, port) = (
        env::get_or_default("HOST", "0.0.0.0"),
        env::get_or_default("PORT", "8080"),
    );

    let addr = format!("{host}:{port}");
    info!("Starting server on {addr}");

    HttpServer::new(move || {
        App::new()
            .configure(|cfg| router::init(pg.clone(), rd.clone(), email_client.clone(), cfg))
            .wrap(http::cors::setup_cors(&["127.0.0.1"], &["test-header"]))
            .wrap(http::security_headers::default())
            .wrap(Logger::default())
    })
    .bind(addr)?
    .run()
    .await
}
