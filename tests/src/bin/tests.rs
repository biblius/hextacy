//! Runs all tests with tracing
use env_logger::fmt::Color;
use env_logger::Env;
use std::env;
use std::io::Write;
use tests::{actors, config};

pub fn main() {
    // Set up the logger for debugging
    env::set_var("TRACING_LEVEL", "debug");
    env_logger::Builder::from_env(Env::default().filter("TRACING_LEVEL"))
        .format(|buf, record| {
            let mut style = buf.style();
            style.set_color(Color::White);
            writeln!(buf, "{}", style.value(record.args()))
        })
        .init();

    // Run tests
    actors::direct_message_handling::simple_message_handling().unwrap();
    actors::direct_message_handling::simple_broadcast().unwrap();
    actors::broker_test::add_sub();
    actors::broker_test::handle_subscribe();
    config::load_env();
    config::set_env_vars();
}
