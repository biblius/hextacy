use actix_web::{
    web::{self, Data},
    App, HttpServer,
};
use infrastructure::{
    config::{env, logger},
    http,
};
use tracing::info;
mod api;
mod error;
mod middleware;
mod models;
mod state;
use api::router;

pub async fn hello_world() -> impl actix_web::Responder {
    "Sanity works!"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env::load_from_file("./.env").unwrap();

    logger::init("info");
    //logger::init_file("trace", "server.log");

    let state = Data::new(state::State::init());

    let (host, port) = (
        env::get_or_default("HOST", "0.0.0.0"),
        env::get_or_default("PORT", "8080"),
    );

    infrastructure::email::send_email(
        Some("Yolo mcSwag"),
        "biblius khan",
        "josip.benkodakovic@barrage.net",
        "Desiii",
        "Tusmoooo".to_string(),
    )
    .unwrap();

    let addr = format!("{}:{}", host, port);
    info!("Starting server on {}:{}", host, port);

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .configure(router::set_routes)
            .route("/hello", web::get().to(hello_world))
            .wrap(http::cors::setup_cors(&["127.0.0.1"], &["test-header"]))
            .wrap(http::security_headers::default())
    })
    .bind(addr)?
    .run()
    .await
}
