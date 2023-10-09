mod cache;
mod config;
mod controllers;
mod core;
mod db;
mod error;

use config::state::AppState;
use error::Error;
use hextacy::env;

pub type AppResult<T> = Result<T, Error>;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let level = std::env::args().nth(1);
    let level = level.as_deref().unwrap_or("debug");
    hextacy::logger::init(level);

    env::load_from_file("examples/template/.env").unwrap();

    let state = AppState::init().await;

    config::http::start_server(state).await
}
