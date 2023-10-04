use super::{crypto::Crypto, envex::EnvExOptions};
use clap::{Parser, Subcommand};
use std::fmt::Display;

#[derive(Debug, Parser)]
#[command(author, version = "0.1", about, long_about = None)]
pub struct Xtc {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    // .env.example
    Envex(EnvExOptions),

    // crypto utils
    Crypto(Crypto),
    C(Crypto),

    // start interactive
    Interactive,
    I,

    Init,
}

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::Envex(_) => write!(f, "Generating .env.example"),
            Command::C(_) | Command::Crypto(_) => write!(f, "Cryptographying"),
            Command::Interactive | Command::I => write!(f, "Initiating interactive session"),
            Command::Init => write!(f, "Initialising 6tc template"),
        }
    }
}
