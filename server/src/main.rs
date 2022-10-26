mod api;
mod error;
mod services;

use actix_web::{middleware::Logger, App, HttpServer};
use api::router;
use infrastructure::{
    clients::{
        email,
        store::{postgres::Postgres, redis::Redis},
    },
    config::{env, logger},
    web::http,
};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use std::sync::Arc;
use tracing::info;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env::load_from_file("./.env").unwrap();

    logger::init("debug");
    // logger::init_file("debug", "server.log");

    let pg = Arc::new(Postgres::new());
    info!("Postgres pool initialized");

    let rd = Arc::new(Redis::new());
    info!("Redis pool initialized");

    let email_client = Arc::new(email::build_client());
    info!("Email client initialized");

    let (host, port) = (
        env::get_or_default("HOST", "0.0.0.0"),
        env::get_or_default("PORT", "8080"),
    );

    let addr = format!("{host}:{port}");
    info!("Starting server on {addr}");

    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("./openssl/key.pem", SslFiletype::PEM)
        .unwrap();
    builder
        .set_certificate_chain_file("./openssl/cert.pem")
        .unwrap();

    HttpServer::new(move || {
        App::new()
            .configure(|cfg| router::init(pg.clone(), rd.clone(), email_client.clone(), cfg))
            .wrap(http::cors::setup_cors(&["127.0.0.1"], &["test-header"]))
            .wrap(http::security_headers::default())
            .wrap(Logger::default())
    })
    .bind_openssl(addr, builder)?
    .run()
    .await
}
