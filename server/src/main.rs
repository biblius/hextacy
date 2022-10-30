mod api;
mod configure;
mod error;
mod helpers;

use actix_web::{middleware::Logger, App, HttpServer};
use infrastructure::{
    config::{env, logger},
    web::http,
};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use tracing::info;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env::load_from_file("./.env").unwrap();

    logger::init("debug");
    // logger::init_file("debug", "server.log");

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
            .configure(configure::configure)
            .wrap(http::cors::setup_cors(&["127.0.0.1"], &["test-header"]))
            .wrap(http::security_headers::default())
            .wrap(Logger::default())
    })
    .bind_openssl(addr, builder)?
    .run()
    .await
}
