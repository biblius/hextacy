//! Generate an env example file from the .env file in the root
use clap::Args;
use std::fmt::Write;

/// Create an `.env.example` file from the .env file in the root or the specified path
#[derive(Debug, Args)]
pub struct EnvExOptions {
    /// If provided alx will search for a .env file in the given directory and generate a .env.example there
    #[arg(short, long)]
    pub path: Option<String>,
}

pub fn envex(path: Option<String>) {
    let mut path = match path {
        Some(p) => p,
        None => String::from("./.env"),
    };

    let env_file =
        std::fs::read_to_string(&path).expect(&format!("Couldn't load .env file at {}", path));

    let mut example = String::new();

    for l in env_file.lines() {
        if let Some(i) = l.find('=') {
            if l.contains("_URL") {
                writeln!(example, "{}", l).unwrap();
            } else {
                writeln!(example, "{}", l.split_at(i + 1).0).unwrap();
            }
        } else {
            writeln!(example, "{}", l).unwrap();
        }
    }

    write!(path, ".example").unwrap();

    std::fs::write(&path, example).unwrap();
}
