mod app;
mod cache;
mod config;
mod db;
mod error;
mod helpers;
mod services;

use crate::config::{
    constants::{OPEN_SSL_CERT_PATH, OPEN_SSL_KEY_PATH},
    AppState,
};
use actix_web::{
    middleware::{DefaultHeaders, Logger},
    App, HttpServer,
};
use hextacy::env;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use tracing::info;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let log = std::env::args().nth(1);
    let log = log.as_deref().unwrap_or("debug");

    info!("Starting with: {:?}", std::env::args());

    let env = std::env::args().nth(2);
    let env_path = env.as_deref().unwrap_or("examples/server");

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

    let state = AppState::init().await.unwrap();

    HttpServer::new(move || {
        App::new()
            .configure(|cfg| app::router::route(&state, cfg))
            .service(actix_web::web::resource("/hello").to(|| async { "OK" }))
            .wrap(config::cors::setup_cors(&["127.0.0.1"], &["test-header"]))
            .wrap(security_headers())
            .wrap(Logger::default())
    })
    .bind_openssl(addr, builder)?
    .run()
    .await
}

/// Builds the default security header middleware
pub fn security_headers() -> DefaultHeaders {
    use hextacy::web::xhttp::security_headers::*;
    DefaultHeaders::new()
        .add(default_content_security_policy())
        .add(cross_origin_embedder_policy("require-corp"))
        .add(cross_origin_opener_policy("same-origin"))
        .add(cross_origin_resource_policy("same-origin"))
        .add(referrer_policy(&["no-referrer", "same-origin"]))
        .add(strict_transport_security(
            31536000, // 1 year
            Some("includeSubDomains"),
        ))
        .add(no_sniff())
        .add(dns_prefetch_control(false))
        .add(ie_no_open())
        .add(frame_options(true))
        .add(cross_domain_policies("none"))
        .add(xss_filter(false))
}
