use std::fs;

use clap::{Args, Subcommand};
use colored::Colorize;

use crate::{
    boiler::{
        self,
        files::{handle_create_dir, write_to_mod_file},
    },
    uppercase, DEFAULT_PATH, FILES,
};

/// Router commands
#[derive(Debug, Args)]
pub(crate) struct RouteCommand {
    #[clap(subcommand)]
    pub command: RouteSubcommand,
}
/// Commands for generating and modifying route endpoints.
#[derive(Debug, Subcommand)]
pub(crate) enum RouteSubcommand {
    /// Generate a route.
    Gen(RouteArgs),
    /// Shorthand for generate.
    G(RouteArgs),
    /// Add a api to an existing route endpoint.
    AddApi(RouteName),
    /// Shorthand for add api.
    AC(RouteName),
}

/// Contains endpoint information
#[derive(Debug, Args)]
pub(crate) struct RouteArgs {
    /// The name of the router endpoint.
    pub name: String,
    #[arg(short, long)]
    /// The various services or repositories the endpoint will use. Comma seperated.
    pub apis: Option<String>,
    #[arg(short, long)]
    /// The path to the API you wish to generate this endpoint. Defaults to ./server/api/router
    pub path: Option<String>,
}

#[derive(Debug, Args)]
pub(crate) struct RouteName {
    /// The name of a router endpoint
    pub name: String,
}

pub(crate) fn handle_gen_route(args: RouteArgs) {
    let mut path = format!("{}/{}", DEFAULT_PATH, args.name);
    // If a path is given switch to it
    if let Some(ref p) = args.path {
        path = format!("{}/{}", p, args.name)
    }
    let path = &path;

    let service_name = uppercase(&args.name);

    let router_mod_path = format!("{}/mod.rs", DEFAULT_PATH);

    // Gather up apis if any
    let apis = match args.apis {
        Some(ref c) => c.split(',').collect::<Vec<&str>>(),
        None => vec![],
    };

    // Try to create the directory and prompt for overwrite if it exists
    if !handle_create_dir(path) {
        return;
    }

    // Append the mod clause to the existing router.mod file
    write_to_mod_file(&router_mod_path, &args.name);

    for file in FILES {
        let mut contents = String::new();
        match file {
            "api" => boiler::plate::apis(&mut contents, &apis),
            "domain" => boiler::plate::domain(&mut contents, &service_name, &apis),
            "infrastructure" if !apis.is_empty() => {
                boiler::plate::infrastructure(&mut contents, &apis)
            }
            "setup" => boiler::plate::setup(&mut contents, &service_name, &apis),
            "mod" => boiler::plate::r#mod(&mut contents),
            _ => {}
        }
        fs::write(&format!("{}/{}.rs", path, file), contents.clone())
            .expect("Could't write to file");
        contents.clear();
    }
    println!("{}{}", "Successfully wrote endpoint ".green(), path)
}
