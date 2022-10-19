use actix_web::{middleware::Logger, web, App, HttpServer};
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

pub async fn hello_world() -> impl actix_web::Responder {
    "Sanity works!"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env::load_from_file("./.env").unwrap();

    logger::init("debug");
    // logger::init_file("debug", "server.log");

    //let state = Data::new(state::State::init());

    let pg = Arc::new(Pg::new());
    info!("Postgres pool initialized");

    let rd = Arc::new(Rd::new());
    info!("Redis pool initialized");

    let email_client = Arc::new(email::build_client());
    info!("Redis pool initialized");

    let (host, port) = (
        env::get_or_default("HOST", "0.0.0.0"),
        env::get_or_default("PORT", "8080"),
    );

    let addr = format!("{}:{}", host, port);
    info!("Starting server on {}:{}", host, port);

    /* infrastructure::email::send_email(
        Some("Yolo mcSwag"),
        "biblius khan",
        "josip.benkodakovic@barrage.net",
        "Desiii",
        "Tusmoooo".to_string(),
    )
    .unwrap(); */

    HttpServer::new(move || {
        App::new()
            .configure(|cfg| {
                router::init(
                    Arc::clone(&pg),
                    Arc::clone(&rd),
                    Arc::clone(&email_client),
                    cfg,
                )
            })
            .route("/hello", web::get().to(hello_world))
            .wrap(http::cors::setup_cors(&["127.0.0.1"], &["test-header"]))
            .wrap(http::security_headers::default())
            .wrap(Logger::default())
    })
    .bind(addr)?
    .run()
    .await
}
