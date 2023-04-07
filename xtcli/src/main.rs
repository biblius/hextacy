mod analyzer;
mod boiler;
mod commands;
mod config;
mod error;

use crate::analyzer::analyze;
use crate::commands::crypto::{generate_rsa_key_pair, write_pw, write_secret};
use crate::commands::generate::{handle_gen_mw, handle_gen_route};
use crate::commands::interactive::init_interactive;
use crate::commands::migration::{
    migration_generate, migration_redo, migration_rev, migration_run,
};
use crate::commands::xtc::{Command, Xtc};
use clap::Parser;
use commands::generate::GenerateSubcommand;
use std::sync::atomic::AtomicBool;

pub const INDENT: &str = "    ";
pub const DEFAULT_API_PATH: &str = "server/src/api";
pub const DEFAULT_MIDDLEWARE_PATH: &str = "server/src/api/middleware";
pub const DEFAULT_ROUTER_PATH: &str = "server/src/api/router";
pub const ROUTE_FILES: [&str; 7] = [
    "api",
    "data",
    "domain",
    "handler",
    "infrastructure",
    "setup",
    "mod",
];
pub const MW_FILES: [&str; 5] = ["api", "domain", "infrastructure", "interceptor", "mod"];
static VERBOSE: AtomicBool = AtomicBool::new(false);

pub fn main() {
    let xtc = Xtc::parse();
    println!("{}", xtc.command);
    match xtc.command {
        Command::Generate(cmd) | Command::Gen(cmd) | Command::G(cmd) => match cmd.subject {
            GenerateSubcommand::Route(args) | GenerateSubcommand::R(args) => {
                verbose(args.verbose);
                let path = match args.path {
                    Some(ref p) => p.to_string(),
                    None => DEFAULT_ROUTER_PATH.to_string(),
                };
                handle_gen_route(args, &path);
            }
            GenerateSubcommand::Middleware(args) | GenerateSubcommand::MW(args) => {
                verbose(args.verbose);
                let path = match args.path {
                    Some(ref p) => p.to_string(),
                    None => DEFAULT_MIDDLEWARE_PATH.to_string(),
                };
                handle_gen_mw(args, &path);
            }
        },
        Command::Analyze(args) | Command::Anal(args) | Command::A(args) => {
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
        Command::Migration(sc) | Command::Mig(sc) | Command::M(sc) => match sc.action {
            commands::migration::MigrationSubcommand::Gen(opts) => migration_generate(opts),
            commands::migration::MigrationSubcommand::Run => migration_run(),
            commands::migration::MigrationSubcommand::Rev => migration_rev(),
            commands::migration::MigrationSubcommand::Redo(opts) => migration_redo(opts),
        },
        Command::Crypto(sc) | Command::C(sc) => match sc.action {
            commands::crypto::CryptoSubcommand::PW(opts) => write_pw(opts),
            commands::crypto::CryptoSubcommand::Rsa => {
                generate_rsa_key_pair().expect("RSA Generation error")
            }
            commands::crypto::CryptoSubcommand::Secret(opts) => write_secret(opts),
        },
        Command::Interactive | Command::I => {
            init_interactive().expect("Error occurred in interactive session")
        }
    }
}

fn uppercase(s: &str) -> String {
    format!("{}{}", &s[..1].to_string().to_uppercase(), &s[1..])
}

pub fn print(s: &str) {
    if VERBOSE.load(std::sync::atomic::Ordering::SeqCst) {
        println!("{s}");
    }
}

fn verbose(v: bool) {
    VERBOSE.fetch_or(v, std::sync::atomic::Ordering::SeqCst);
}
