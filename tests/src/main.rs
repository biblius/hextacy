pub mod actors;
pub mod config;
pub mod schema;
pub mod storage;
pub mod websocket;

use env_logger::fmt::Color;
use env_logger::Env;
use std::env;
use std::io::Write;

const ENV_PATH: &'static str = "./tests/.env.test";

pub fn main() {
    // Set up the logger for debugging
    env::set_var("TRACING_LEVEL", "trace");
    env_logger::Builder::from_env(Env::default().filter("TRACING_LEVEL"))
        .format(|buf, record| {
            let mut style = buf.style();
            style.set_color(Color::White);
            writeln!(buf, "{}", style.value(record.args()))
        })
        .init();

    // Config test also set the env
    config::set_env_vars();
    config::load_from_dot_env(ENV_PATH);

    // Actors
    actors::direct_message_handling::simple_message_handling().unwrap();
    actors::direct_message_handling::simple_broadcast().unwrap();
    actors::broker_test::add_sub();
    actors::broker_test::handle_subscribe();

    // Postgres
    storage::establish_pg_connection();
    storage::pg_transaction();
    storage::pg_transaction_fail();

    // Redis
    storage::establish_rd_connection();

    // Mongo
    storage::mongo_insert_with_transaction();
}
