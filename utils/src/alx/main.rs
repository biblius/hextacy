mod analyzer;
mod boiler;
mod commands;
mod config;
mod error;

use crate::analyzer::handle_analyze;
use crate::commands::mw::MiddlewareSubcommand;
use crate::commands::route::{handle_gen_route, RouteSubcommand};
use crate::commands::Command;
use clap::Parser;
use commands::AlxArgs;

pub const INDENT: &str = "    ";
pub const DEFAULT_PATH: &str = "server/src/api/router";
pub const FILES: [&str; 7] = [
    "contract",
    "data",
    "domain",
    "handler",
    "infrastructure",
    "mod",
    "setup",
];

pub fn main() {
    let args = AlxArgs::parse();
    println!("Running with args: {:?}", args);

    match args.command {
        Command::Route(cmd) | Command::R(cmd) => match cmd.command {
            RouteSubcommand::Gen(args) | RouteSubcommand::G(args) => handle_gen_route(args),
            RouteSubcommand::AddContract(route_name) | RouteSubcommand::AC(route_name) => {
                println!("{:?}", route_name)
            }
        },
        Command::Middleware(cmd) | Command::MW(cmd) => match cmd.command {
            MiddlewareSubcommand::Gen(_) | MiddlewareSubcommand::G(_) => todo!(),
            MiddlewareSubcommand::AddContract(_) | MiddlewareSubcommand::AC(_) => todo!(),
        },
        Command::Analyze(options) | Command::Anal(options) => handle_analyze(options),
    }
}

fn uppercase(s: &str) -> String {
    format!("{}{}", &s[..1].to_string().to_uppercase(), &s[1..])
}
