mod boiler;
mod commands;
mod config;
mod error;

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

const XTC_DIR_VAR: &str = "XTC_ACTIVE_DIRECTORY";

fn handle_working_dir() -> String {
    std::env::var(XTC_DIR_VAR).unwrap_or_else(|_| {
        enter_working_dir();
        std::env::var(XTC_DIR_VAR).unwrap()
    })
}

fn enter_working_dir() {
    let current = std::env::current_dir().unwrap();
    loop {
        println!("Enter active directory path: [{}/]", current.display());
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        if input == "q" {
            std::process::exit(0);
        }
        let path = format!("{}/{input}", current.display());
        if std::fs::read_dir(input).is_ok() {
            std::env::set_var(XTC_DIR_VAR, path);
            break;
        }
        println!("Invalid directory set, please enter a valid directory or press q to quit")
    }
    println!(
        "Successfully set xtc active directory to: {}",
        std::env::var(XTC_DIR_VAR).unwrap()
    );
}

pub const INDENT: &str = "    ";
// pub const DEFAULT_API_PATH: &str = "server/src/api";
pub const ROUTE_FILES: [&str; 6] = ["data", "service", "handler", "adapters", "setup", "mod"];
pub const MW_FILES: [&str; 2] = ["interceptor", "adapter"];
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
                    None => handle_working_dir(),
                };
                handle_gen_route(args, &path);
            }
            GenerateSubcommand::Middleware(args) | GenerateSubcommand::MW(args) => {
                verbose(args.verbose);
                let path = match args.path {
                    Some(ref p) => p.to_string(),
                    None => handle_working_dir(),
                };
                handle_gen_mw(args, &path);
            }
        },
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
