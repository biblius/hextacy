use std::fmt::Display;

use super::generate::GenerateSubject;
use crate::analyzer::analyze::AnalyzeOptions;
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(author, version = "0.1", about, long_about = None)]
pub struct Alx {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Generate(GenerateSubject),
    G(GenerateSubject),

    Analyze(AnalyzeOptions),
    Anal(AnalyzeOptions),
}

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::Generate(s) | Command::G(s) => match s.subject {
                super::generate::GenSubject::Route(_) | super::generate::GenSubject::R(_) => {
                    write!(f, "generate route")
                }
                super::generate::GenSubject::Middleware(_) | super::generate::GenSubject::MW(_) => {
                    write!(f, "generate middleware")
                }
            },
            Command::Analyze(_) | Command::Anal(_) => write!(f, "analyze"),
        }
    }
}
