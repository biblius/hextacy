//! Runs all tests with tracing
use env_logger::fmt::Color;
use env_logger::Env;
use std::env;
use std::io::Write;
use tests::mycro_actors;

pub fn main() {
    env::set_var("TRACING_LEVEL", "debug");
    env_logger::Builder::from_env(Env::default().filter("TRACING_LEVEL"))
        .format(|buf, record| {
            let mut style = buf.style();
            style.set_color(Color::White);
            writeln!(buf, "{}", style.value(record.args()))
        })
        .init();
    mycro_actors::broker_test::add_sub();
    mycro_actors::broker_test::handle_subscribe();
}
