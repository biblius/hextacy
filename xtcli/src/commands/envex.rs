//! Generate an env example file from the .env file in the root
use clap::Args;
use std::fmt::Write;

/// Create an `.env.example` file from the .env file in the root or the specified path
#[derive(Debug, Args)]
pub struct EnvExOptions {
    /// If provided xtc will search for a .env file in the given directory and generate a .env.example there
    #[arg(short, long)]
    pub path: Option<String>,
}

pub fn envex(path: Option<String>) {
    let mut path = match path {
        Some(p) => p,
        None => String::from("./.env"),
    };

    let env_file = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("Couldn't load .env file at {path}"));

    let mut example = String::new();

    for line in env_file.lines() {
        if let Some(i) = line.find('=') {
            if line.contains("_URL") {
                writeln!(example, "{line}").unwrap();
            } else {
                writeln!(example, "{}", line.split_at(i + 1).0).unwrap();
            }
        } else {
            writeln!(example, "{line}").unwrap();
        }
    }

    write!(path, ".example").unwrap();

    std::fs::write(&path, example).unwrap();
}
