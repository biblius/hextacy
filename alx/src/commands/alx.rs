use super::{envex::EnvExOptions, generate::GenerateSubject};
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
    Generate(GenerateSubject),
    Gen(GenerateSubject),
    G(GenerateSubject),

    Analyze(AnalyzeOptions),
    Anal(AnalyzeOptions),

    Envex(EnvExOptions),
}

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::Generate(s) | Command::Gen(s) | Command::G(s) => match s.subject {
                super::generate::GenSubject::Route(_) | super::generate::GenSubject::R(_) => {
                    write!(f, "Generating route")
                }
                super::generate::GenSubject::Middleware(_) | super::generate::GenSubject::MW(_) => {
                    write!(f, "Generating middleware")
                }
            },
            Command::Analyze(_) | Command::Anal(_) => write!(f, "Analyzing"),
            Command::Envex(_) => write!(f, "Generating .env.example"),
        }
    }
}
