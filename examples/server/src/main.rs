mod api;
mod config;
mod db;
mod error;
mod helpers;
mod services;

use crate::config::{
    constants::{OPEN_SSL_CERT_PATH, OPEN_SSL_KEY_PATH},
    AppState,
};
use actix_web::{middleware::Logger, App, HttpServer};
use hextacy::{env, web::http};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use tracing::info;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let log = std::env::args().nth(1);
    let log = log.as_ref().map(|l| l.as_str()).unwrap_or("debug");

    let env = std::env::args().nth(2);
    let env_path = env
        .as_ref()
        .map(|p| p.as_str())
        .unwrap_or("examples/server");

    env::load_from_file(&format!("{env_path}/.env")).unwrap();

    hextacy::logger::init(log);
    // hextacy::logger::init_file(log, "server.log");

    // Init all the lazy loaded static stuff
    helpers::resources::initialize();

    let (host, port) = (
        env::get_or_default("HOST", "0.0.0.0"),
        env::get_or_default("PORT", "8080"),
    );

    let addr = format!("{host}:{port}");
    info!("Starting server on {addr}");

    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file(OPEN_SSL_KEY_PATH, SslFiletype::PEM)
        .unwrap();
    builder
        .set_certificate_chain_file(OPEN_SSL_CERT_PATH)
        .unwrap();

    let state = AppState::init().await;

    HttpServer::new(move || {
        App::new()
            .configure(|cfg| config::init(cfg, &state))
            .wrap(config::cors::setup_cors(&["127.0.0.1"], &["test-header"]))
            .wrap(http::security_headers::default())
            .wrap(Logger::default())
    })
    .bind_openssl(addr, builder)?
    .run()
    .await
}