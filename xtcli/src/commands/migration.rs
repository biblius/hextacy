use crate::print;
use clap::{Args, Subcommand};
use std::{
    io::{stdin, Write},
    process::{Command, Stdio},
};

/// Generate, run or reverse migrations
#[derive(Debug, Args)]
pub struct Migration {
    #[clap(subcommand)]
    pub action: MigrationSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum MigrationSubcommand {
    /// Generate a new migration
    Gen(GenMigration),
    /// Run pending migrations
    Run,
    /// Reverse a migration
    Rev,
    /// Redo migrations
    Redo(RedoMigration),
}

#[derive(Debug, Args, Default, Clone)]
/// Migration arguments
pub struct RedoMigration {
    /// If given this will redo all migrations
    #[arg(long, short, action)]
    pub all: bool,
}

#[derive(Debug, Args, Default, Clone)]
/// Migration arguments
pub struct GenMigration {
    /// Migration name
    pub name: String,
}

pub fn migration_generate(args: GenMigration) {
    let abs = get_absolute_migration_path();
    std::env::set_current_dir(abs).expect("Couldn't set env");
    let name = &args.name;
    let out = Command::new("diesel")
        .args(["migration", "generate", name])
        .output()
        .expect("Failed to execute `diesel migration generate`");
    std::io::stdout().write_all(&out.stdout).unwrap();
    println!("Successfully generated migration {name}")
}

pub fn migration_run() {
    handle_db_url();
    let abs = get_absolute_migration_path();
    let out = Command::new("diesel")
        .args(["migration", "run", "--migration-dir", &abs])
        .output()
        .expect("Failed to execute `diesel migration run`");
    if !out.stdout.is_empty() {
        std::io::stdout().write_all(&out.stdout).unwrap();
    }
    if !out.stderr.is_empty() {
        std::io::stderr().write_all(&out.stderr).unwrap();
    }
}

pub fn migration_rev() {
    handle_db_url();
    let abs = get_absolute_migration_path();
    let out = Command::new("diesel")
        .args(["migration", "revert", "--migration-dir", &abs])
        .output()
        .unwrap();
    if !out.stdout.is_empty() {
        std::io::stdout().write_all(&out.stdout).unwrap();
    }
    if !out.stderr.is_empty() {
        std::io::stderr().write_all(&out.stderr).unwrap();
    }
}

pub fn migration_redo(redo: RedoMigration) {
    handle_db_url();
    let abs = get_absolute_migration_path();
    std::env::set_current_dir(abs).expect("Couldn't set env");

    let mut args = vec!["migration", "redo"];
    if redo.all {
        args.push("--all");
    }

    Command::new("diesel")
        .args(args)
        .stdout(Stdio::inherit())
        .output()
        .unwrap();
    println!("Successfully restarted migration")
}

/// First tries to load a .env file in the root, then sets the `DATABASE_URL` env variable to the
/// `DATABASE_URL` found in the env file if successful. If unsuccessful it will prompt the user to enter
/// a database name and use the default postgres url.
fn handle_db_url() {
    let env_ok = dotenv::dotenv().is_ok();
    if !env_ok {
        println!("Couldn't load .env file, using default postgres configuration")
    } else {
        let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");
        std::env::set_var("DATABASE_URL", db_url);
        println!("Successfully loaded DATABASE_URL");
    }

    if std::env::var("DATABASE_URL").is_err() {
        println!("Enter the database name you want to run the migrations in:");
        let mut buf = String::new();
        stdin().read_line(&mut buf).expect("Couldn't read line");
        let db_url = format!("postgres://postgres:postgres@localhost:5432/{}", buf.trim());
        std::env::set_var("DATABASE_URL", db_url);
    }
}

/// Gets the absolute path of the directory where diesel.toml is located. Used to set process' working directory.
fn get_absolute_migration_path() -> String {
    // Grab the current directory
    let pwd = Command::new("pwd").output().unwrap().stdout.to_vec();
    let current_dir = String::from_utf8(pwd).unwrap();
    print(&format!(
        "Searching for migration directory in {current_dir}",
    ));

    // Find the diesel toml file
    let mig_dir = Command::new("find")
        .args([".", "-name", "migrations", "-type", "d"])
        .output()
        .unwrap()
        .stdout
        .to_vec();

    if mig_dir.is_empty() {
        panic!("Migrations directory not found")
    }

    let path = String::from_utf8(mig_dir[1..].to_vec()).unwrap();
    println!("Found migrations directory at: {path}");

    format!("{}{}", current_dir.trim(), path.trim())
}
