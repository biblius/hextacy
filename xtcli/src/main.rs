mod commands;
mod error;

use crate::commands::crypto::{generate_rsa_key_pair, write_pw, write_secret};
use crate::commands::interactive::init_interactive;
use crate::commands::xtc::{Command, Xtc};
use clap::Parser;
use reqwest::header;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

static VERBOSE: AtomicBool = AtomicBool::new(false);

pub fn main() -> Result<(), std::io::Error> {
    let xtc = Xtc::parse();
    println!("{}", xtc.command);
    match xtc.command {
        Command::Envex(args) => {
            commands::envex::envex(args.path);
        }
        Command::Crypto(sc) | Command::C(sc) => match sc.action {
            commands::crypto::CryptoSubcommand::PW(opts) => write_pw(opts),
            commands::crypto::CryptoSubcommand::Rsa => {
                generate_rsa_key_pair().expect("RSA Generation error")
            }
            commands::crypto::CryptoSubcommand::Secret(opts) => write_secret(opts),
        },
        Command::Interactive | Command::I => {
            // init_interactive().expect("Error occurred in interactive session")
        }
        Command::Init => {
            let mut dir = std::env::current_dir()?;
            dir.push("temp");

            let res = reqwest::blocking::Client::new()
                .get("https://api.github.com/repos/biblius/hxtc_template/tarball/")
                .header(header::ACCEPT, "application/vnd.github+json")
                .header("X-GitHub-Api-Version", "2022-11-28")
                .header(header::USER_AGENT, "hextacy")
                .send()
                .unwrap();

            std::fs::create_dir(&dir)?;
            let tarb = flate2::read::GzDecoder::new(res);
            let mut archive = tar::Archive::new(tarb);
            archive.unpack(&dir)?;

            let src = find_src(&dir).expect("malformed template");
            move_dir_all(src, "template/src")?;
            fs::remove_dir_all(dir)?;
        }
    }
    Ok(())
}

#[derive(Debug)]
struct InitArgs {
    /// The name of the project that will be created
    name: String,

    /// The path to the directory where the project will be initialised
    path: String,
}

/// Solely used by the init command to find the src directory of the extracted repo tarball
fn find_src(path: &PathBuf) -> Option<PathBuf> {
    let dir = fs::read_dir(path).ok()?;
    let dir = dir.collect::<Vec<_>>();

    // Checks for the initial dir with everything inside
    if dir
        .iter()
        .filter(|e| e.as_ref().is_ok_and(|e| e.path().is_dir()))
        .collect::<Vec<_>>()
        .len()
        == 1
    {
        return find_src(&dir[0].as_ref().ok()?.path());
    }

    for entry in dir {
        let entry = entry.ok()?;
        if entry.file_name() == "src" {
            return Some(entry.path());
        }
    }

    None
}

fn move_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dst = dst.as_ref().join(entry.file_name());
        if ty.is_dir() {
            move_dir_all(entry.path(), dst)?;
        } else {
            fs::rename(entry.path(), dst)?;
        }
    }
    Ok(())
}

const XTC_DIR_VAR: &str = "XTC_ACTIVE_DIRECTORY";
const DEFAULT_DIR: &str = "hextatic";

fn prompt_dir(current: &str) {
    loop {
        println!("Choose name: {current}/{DEFAULT_DIR}]");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        if input == "q" {
            std::process::exit(0);
        }
        let path = format!("{current}/{input}");
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

#[derive(Debug, Default)]
struct InitState {
    name: String,
}

fn capitalise(s: &str) -> String {
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
