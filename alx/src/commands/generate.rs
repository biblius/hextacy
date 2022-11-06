use crate::{
    boiler::{
        self,
        files::{handle_create_dir, write_to_mod_file},
        plate::BoilerType,
    },
    print, uppercase, MW_FILES, ROUTE_FILES,
};
use clap::{Args, Subcommand};
use colored::Colorize;
use std::fs;

/// Generate a new endpoint or middleware
#[derive(Debug, Args)]
pub struct GenerateSubject {
    #[clap(subcommand)]
    pub subject: GenSubject,
}

/// Commands for generating stuff.
#[derive(Debug, Subcommand)]
pub enum GenSubject {
    /// Generate a route.
    Route(GenerateArgs),
    /// Shorthand for route.
    R(GenerateArgs),
    /// Generate middleware boilerplate.
    Middleware(GenerateArgs),
    /// Shorthand for add contract.
    MW(GenerateArgs),
}

/// Contains endpoint information
#[derive(Debug, Args)]
pub struct GenerateArgs {
    /// The name of the router endpoint.
    pub name: String,
    #[arg(short, long)]
    /// The various services or repositories the endpoint will use. Comma seperated. e.g. `-c repo,cache`
    pub contracts: Option<String>,
    #[arg(short, long)]
    /// The path to the API you wish to generate this endpoint. Defaults to ./server/api/router
    pub path: Option<String>,
    /// Print what's going to std out
    #[arg(short, long)]
    pub verbose: Option<bool>,
}

pub fn handle_gen_route(args: GenerateArgs, router_path: &str) {
    let mut ep_path = format!("{}/{}", router_path, args.name);
    // If a path is given switch to it
    if let Some(ref p) = args.path {
        ep_path = format!("{}/{}", p, args.name)
    }
    let ep_path = &ep_path;

    let service_name = uppercase(&args.name);

    // Gather up contracts if any
    let contracts = match args.contracts {
        Some(ref c) => c.split(',').collect::<Vec<&str>>(),
        None => vec![],
    };

    // Try to create the directory and prompt for overwrite if it exists
    if !handle_create_dir(ep_path) {
        return;
    }

    // Append the mod clause to the existing router.mod file
    let router_mod = format!("{}/mod.rs", router_path);
    write_to_mod_file(&router_mod, &args.name);

    for file in ROUTE_FILES {
        print(&format!("{} Writing {}.rs", "\u{270E}".blue(), file));
        let mut contents = String::new();
        match file {
            "contract" => boiler::plate::contracts(&mut contents, &contracts, BoilerType::Route),
            "domain" => boiler::plate::domain(&mut contents, &service_name, &contracts),
            "infrastructure" if !contracts.is_empty() => {
                boiler::plate::infrastructure(&mut contents, &contracts)
            }
            "setup" => boiler::plate::setup(&mut contents, &service_name, &contracts),
            "mod" => boiler::plate::r#mod(&mut contents, BoilerType::Route),
            _ => {}
        }
        fs::write(&format!("{}/{}.rs", ep_path, file), contents.clone())
            .expect("Could't write to file");
        contents.clear();
    }
    print(&format!(
        "{}{}",
        "Successfully wrote route ".green(),
        ep_path
    ))
}

pub fn handle_gen_mw(args: GenerateArgs, mw_path: &str) {
    let mut ep_path = format!("{}/{}", mw_path, args.name);
    // If a path is given switch to it
    if let Some(ref p) = args.path {
        ep_path = format!("{}/{}", p, args.name)
    }
    let ep_path = &ep_path;

    let service_name = uppercase(&args.name);

    // Gather up contracts if any
    let contracts = match args.contracts {
        Some(ref c) => c.split(',').collect::<Vec<&str>>(),
        None => vec![],
    };

    // Try to create the directory and prompt for overwrite if it exists
    if !handle_create_dir(ep_path) {
        return;
    }

    // Append the mod clause to the existing router.mod file
    let mw_mod = format!("{}/mod.rs", mw_path);
    write_to_mod_file(&mw_mod, &args.name);

    for file in MW_FILES {
        print(&format!("{} Writing {}.rs", "\u{270E}".blue(), file));
        let mut contents = String::new();
        match file {
            "contract" => boiler::plate::contracts(&mut contents, &contracts, BoilerType::MW),
            "domain" => boiler::plate::domain(&mut contents, &service_name, &contracts),
            "infrastructure" if !contracts.is_empty() => {
                boiler::plate::infrastructure(&mut contents, &contracts)
            }
            "interceptor" => {
                boiler::plate::mw_interceptor(&mut contents, &service_name, &contracts)
            }
            "mod" => boiler::plate::r#mod(&mut contents, BoilerType::MW),
            _ => {}
        }
        fs::write(&format!("{}/{}.rs", ep_path, file), contents.clone())
            .expect("Could't write to file");
        contents.clear();
    }
    print(&format!(
        "{}{}",
        "Successfully wrote middleware ".green(),
        ep_path
    ))
}
