use crate::{
    boiler::{
        self,
        files::{handle_create_dir, write_to_mod_file},
        BoilerType,
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
    pub subject: GenerateSubcommand,
}

/// Commands for generating stuff.
#[derive(Debug, Subcommand)]
pub enum GenerateSubcommand {
    /// Generate a route.
    Route(GenerateArgs),
    /// Shorthand for route.
    R(GenerateArgs),
    /// Generate middleware boilerplate.
    Middleware(GenerateArgs),
    /// Shorthand for add api.
    MW(GenerateArgs),
}

/// Generate arguments
#[derive(Debug, Args, Default, Clone)]
pub struct GenerateArgs {
    /// The name of the route of middleware.
    pub name: String,
    /// The various services or repositories the endpoint will use. Comma seperated. e.g. `-c repository,cache`
    #[arg(short, long)]
    pub components: Option<String>,
    /// The path to the API you wish to generate this endpoint. Defaults to ./server/api/router
    #[arg(short, long)]
    pub path: Option<String>,
    /// Print what's going on to stdout
    #[arg(short, long, action)]
    pub verbose: bool,
}

/// Generate route boilerplate
pub fn handle_gen_route(args: GenerateArgs, router_path: &str) {
    let mut ep_path = format!("{router_path}/{}", args.name);
    // If a path is given switch to it
    if let Some(ref path) = args.path {
        ep_path = format!("{path}/{}", args.name)
    }
    let ep_path = &ep_path;

    let service_name = uppercase(&args.name);

    // Gather up components if any
    let components = match args.components {
        Some(ref c) => c.split(',').collect::<Vec<&str>>(),
        None => vec![],
    };

    // Try to create the directory and prompt for overwrite if it exists
    if !handle_create_dir(ep_path) {
        return;
    }

    // Append the mod clause to the existing router.mod file
    let router_mod = format!("{router_path}/mod.rs");
    write_to_mod_file(&router_mod, &args.name);

    for file in ROUTE_FILES {
        print(&format!("{} Writing {file}.rs", "\u{270E}".blue()));
        let mut contents = String::new();
        match file {
            "api" => boiler::components(&mut contents, &components, BoilerType::Route),
            "domain" => boiler::domain(&mut contents, &service_name, &components),
            "infrastructure" if !components.is_empty() => {
                boiler::infrastructure(&mut contents, &components)
            }
            "setup" => boiler::router::setup(&mut contents, &service_name, &components),
            "mod" => boiler::r#mod(&mut contents, BoilerType::Route),
            _ => {}
        }
        fs::write(format!("{ep_path}/{file}.rs"), contents.clone()).expect("Could't write to file");
        contents.clear();
    }
    print(&format!("{}{ep_path}", "Successfully wrote route ".green(),))
}

/// Generate middleware boilerplate
pub fn handle_gen_mw(args: GenerateArgs, mw_path: &str) {
    let mut ep_path = format!("{mw_path}/{}", args.name);
    // If a path is given switch to it
    if let Some(ref path) = args.path {
        ep_path = format!("{path}/{}", args.name)
    }
    let ep_path = &ep_path;

    let service_name = uppercase(&args.name);

    // Gather up components if any
    let components = match args.components {
        Some(ref c) => c.split(',').collect::<Vec<&str>>(),
        None => vec![],
    };

    // Try to create the directory and prompt for overwrite if it exists
    if !handle_create_dir(ep_path) {
        return;
    }

    // Append the mod clause to the existing router.mod file
    let mw_mod = format!("{mw_path}/mod.rs");
    write_to_mod_file(&mw_mod, &args.name);

    for file in MW_FILES {
        print(&format!("{} Writing {file}.rs", "\u{270E}".blue()));
        let mut contents = String::new();
        match file {
            "api" => boiler::components(&mut contents, &components, BoilerType::MW),
            "domain" => boiler::domain(&mut contents, &service_name, &components),
            "infrastructure" if !components.is_empty() => {
                boiler::infrastructure(&mut contents, &components)
            }
            "interceptor" => {
                boiler::middleware::mw_interceptor(&mut contents, &service_name, &components)
            }
            "mod" => boiler::r#mod(&mut contents, BoilerType::MW),
            _ => {}
        }
        fs::write(format!("{ep_path}/{file}.rs"), contents.clone()).expect("Could't write to file");
        contents.clear();
    }
    print(&format!(
        "{}{ep_path}",
        "Successfully wrote middleware ".green(),
    ))
}
