use super::{crypto::Crypto, envex::EnvExOptions, generate::GenerateSubject, migration::Migration};
use crate::analyzer::analyze::AnalyzeOptions;
use clap::{Parser, Subcommand};
use std::fmt::Display;

#[derive(Debug, Parser)]
#[command(author, version = "0.1", about, long_about = None)]
pub struct Alx {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    // alx component generation
    Generate(GenerateSubject),
    Gen(GenerateSubject),
    G(GenerateSubject),

    // analyzer
    Analyze(AnalyzeOptions),
    Anal(AnalyzeOptions),
    A(AnalyzeOptions),

    // .env.example
    Envex(EnvExOptions),

    // postgres migrations
    Migration(Migration),
    Mig(Migration),
    M(Migration),

    Crypto(Crypto),
    C(Crypto),
}

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::Generate(s) | Command::Gen(s) | Command::G(s) => match s.subject {
                super::generate::GenerateSubcommand::Route(_)
                | super::generate::GenerateSubcommand::R(_) => {
                    write!(f, "Generating route")
                }
                super::generate::GenerateSubcommand::Middleware(_)
                | super::generate::GenerateSubcommand::MW(_) => {
                    write!(f, "Generating middleware")
                }
            },
            Command::Analyze(_) | Command::Anal(_) | Command::A(_) => write!(f, "Analyzing"),
            Command::Envex(_) => write!(f, "Generating .env.example"),
            Command::Migration(c) | Command::Mig(c) | Command::M(c) => match c.action {
                super::migration::MigrationSubcommand::Gen(_) => write!(f, "Generating migration"),
                super::migration::MigrationSubcommand::Run => write!(f, "Running migrations"),
                super::migration::MigrationSubcommand::Rev => write!(f, "Reversing migration"),
                super::migration::MigrationSubcommand::Redo(_) => write!(f, "Restarting migration"),
            },
            Command::C(_) | Command::Crypto(_) => write!(f, "Cryptographying"),
        }
    }
}
