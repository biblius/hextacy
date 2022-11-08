mod analyzer;
mod boiler;
mod commands;
mod config;
mod error;

use crate::analyzer::analyze;
use crate::commands::alx::{Alx, Command};
use crate::commands::generate::{handle_gen_mw, handle_gen_route};
use clap::Parser;
use commands::generate::GenSubject;
use std::sync::atomic::AtomicBool;

pub const INDENT: &str = "    ";
pub const DEFAULT_API_PATH: &str = "server/src/api";
pub const DEFAULT_MIDDLEWARE_PATH: &str = "server/src/api/middleware";
pub const DEFAULT_ROUTER_PATH: &str = "server/src/api/router";
pub const ROUTE_FILES: [&str; 7] = [
    "contract",
    "data",
    "domain",
    "handler",
    "infrastructure",
    "mod",
    "setup",
];
pub const MW_FILES: [&str; 5] = ["contract", "domain", "infrastructure", "interceptor", "mod"];
static VERBOSE: AtomicBool = AtomicBool::new(false);

pub fn main() {
    let alx = Alx::parse();
    println!("{}", alx.command);
    match alx.command {
        Command::Generate(cmd) | Command::Gen(cmd) | Command::G(cmd) => match cmd.subject {
            GenSubject::Route(args) | GenSubject::R(args) => {
                verbose(args.verbose);
                let path = match args.path {
                    Some(ref p) => p.to_string(),
                    None => DEFAULT_ROUTER_PATH.to_string(),
                };
                handle_gen_route(args, &path);
            }
            GenSubject::Middleware(args) | GenSubject::MW(args) => {
                verbose(args.verbose);
                let path = match args.path {
                    Some(ref p) => p.to_string(),
                    None => DEFAULT_MIDDLEWARE_PATH.to_string(),
                };
                handle_gen_mw(args, &path);
            }
        },
        Command::Analyze(args) | Command::Anal(args) => {
            verbose(args.verbose);
            let path = match args.path {
                Some(ref p) => p.to_string(),
                None => DEFAULT_API_PATH.to_string(),
            };
            analyze::handle(args, &path);
        }
        Command::Envex(args) => {
            commands::envex::envex(args.path);
        }
    }
}

fn uppercase(s: &str) -> String {
    format!("{}{}", &s[..1].to_string().to_uppercase(), &s[1..])
}

#[inline]
pub fn print(s: &str) {
    if VERBOSE.fetch_and(true, std::sync::atomic::Ordering::SeqCst) {
        println!("{s}");
    }
}

fn verbose(v: Option<bool>) {
    if let Some(v) = v {
        VERBOSE.fetch_or(v, std::sync::atomic::Ordering::SeqCst);
    }
}
