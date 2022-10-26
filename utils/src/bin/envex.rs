//! Generate an env example file from the .env file in the root
use std::fmt::Write;

/// envex
fn main() {
    let env_file = std::fs::read_to_string(std::path::Path::new("./.env")).unwrap();

    let mut example = String::new();

    for l in env_file.lines() {
        if let Some(i) = l.find('=') {
            writeln!(example, "{}", l.split_at(i + 1).0).unwrap();
        } else {
            writeln!(example, "{}", l).unwrap();
        }
    }

    std::fs::write(std::path::Path::new("./.env.example"), example).unwrap();
}
