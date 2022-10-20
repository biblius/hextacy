use super::env;
use env_logger::fmt::Color;
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
    Config,
};
use std::io::Write;
use tracing::log::{Level, LevelFilter};

pub fn init(level: &str) {
    match level {
        "info" | "INFO" | "debug" | "DEBUG" | "trace" | "TRACE" | "error" | "ERROR" | "warn"
        | "WARN" | "off" | "OFF" => env::set("RUST_LOG", level),
        _ => env::set("RUST_LOG", "info"),
    };

    env_logger::builder()
        .format_timestamp_secs()
        .format_target(true)
        .format_suffix("\n")
        .format(|buf, record| {
            let mut style = buf.style();

            match record.level() {
                Level::Error => style.set_color(Color::Red),
                Level::Warn => style.set_color(Color::Yellow),
                Level::Info => style.set_color(Color::Green),
                Level::Debug => style.set_color(Color::Rgb(200, 200, 200)),
                Level::Trace => style.set_color(Color::Rgb(170, 170, 170)),
            };

            writeln!(
                buf,
                "{} | {} | {} | {}",
                &chrono::Utc::now().to_string().replace('T', " ")[0..21],
                format_args!("{:^5}", style.value(record.level())),
                format_args!("{:^50}", record.target()),
                record.args(),
            )
        })
        .init()
}

/// Initiates a logger that logs to the provided file
///
/// Follow [this link](https://docs.rs/log4rs/latest/log4rs/encode/pattern/index.html) to see the
/// pattern encoder syntax
pub fn init_file(level: &str, path: &str) {
    let level = match level {
        "info" | "INFO" => LevelFilter::Info,
        "debug" | "DEBUG" => LevelFilter::Debug,
        "trace" | "TRACE" => LevelFilter::Trace,
        "error" | "ERROR" => LevelFilter::Error,
        "warn" | "WARN" => LevelFilter::Warn,
        "off" | "OFF" => LevelFilter::Off,
        _ => LevelFilter::Info,
    };

    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{l: <5} | {d(%Y-%m-%d %H:%M:%S)} | {M: <30} | {m}\n",
        )))
        .build(path)
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(level))
        .expect("Couldn't build config");

    log4rs::init_config(config).expect("Couldn't load log4rs");
}
