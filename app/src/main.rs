use actix_web::{
    middleware::Logger,
    web::{self, Data},
    App, HttpServer,
};

use infrastructure::{config, http};
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
    //Initialize the application state
    let state = Data::new(state::AppState::init());
    let (host, port) = (config::get("HOST").unwrap(), config::get("PORT").unwrap());
    let addr = format!("{}:{}", host, port);
    info!("Starting server on {}:{}", host, port);

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .configure(router::set_routes)
            .route("/hello", web::get().to(hello_world))
            .wrap(http::cors::setup_cors(&["127.0.0.1"], &[""]))
            .wrap(http::security_headers::default())
            .wrap(Logger::default())
    })
    .bind(addr)?
    .run()
    .await
}
