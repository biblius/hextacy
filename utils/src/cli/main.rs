mod commands;
mod config;
mod error;

use crate::commands::analyze::{parse, router_read_recursive};
use crate::commands::mw::MiddlewareSubcommand;
use crate::commands::route::RouteSubcommand;
use crate::commands::Command;
use crate::config::ProjectConfig;
use clap::Parser;
use commands::AlxArgs;
use std::fmt::Write;
use std::{fs, path::Path};

const DEFAULT_PATH: &str = "server/src/api/router";

const FILES: [&str; 7] = [
    "contract",
    "data",
    "domain",
    "handler",
    "infrastructure",
    "mod",
    "setup",
];

pub fn main() {
    let args = AlxArgs::parse();
    println!("{:?}", args);

    match args.command {
        Command::Route(cmd) | Command::R(cmd) => match cmd.command {
            RouteSubcommand::Gen(route) | RouteSubcommand::G(route) => {
                let mut path = format!("{}/{}", DEFAULT_PATH, route.name);
                if let Some(ref p) = route.path {
                    path = format!("{}/{}", p, route.name)
                }
                let router_dir = Path::new(&path);

                println!("{:?}", Path::new("./").canonicalize());

                let contracts = match route.contracts {
                    Some(ref c) => c.split(',').collect::<Vec<&str>>(),
                    None => vec![],
                };

                fs::create_dir(router_dir)
                    .map_err(|e| panic!("Could not create directory at path: {}, {}", path, e))
                    .unwrap();

                for file in FILES {
                    let mut contents = String::new();
                    if file == "contract" {
                        // Prepare the contracts if any
                        for s in &contracts {
                            write!(
                                contents,
                                "pub(super) trait {}{}Contract {{\n\t\n}}\n",
                                &s[..1].to_string().to_uppercase(),
                                &s[1..]
                            )
                            .unwrap();
                        }
                    }
                    fs::File::create(&format!("{}/{}.rs", path, file))
                        .expect("Couldn't create file");
                    fs::write(&format!("{}/{}.rs", path, file), contents.clone())
                        .expect("Could't write to file");
                    contents.clear();
                }

                println!("{:?}", router_dir);
                println!("{:?}", route);
            }
            RouteSubcommand::AddContract(route_name) | RouteSubcommand::AC(route_name) => {
                println!("{:?}", route_name)
            }
        },
        Command::Middleware(cmd) | Command::MW(cmd) => match cmd.command {
            MiddlewareSubcommand::Gen(_) | MiddlewareSubcommand::G(_) => todo!(),
            MiddlewareSubcommand::AddContract(_) | MiddlewareSubcommand::AC(_) => todo!(),
        },
        Command::Analyze | Command::Anal => {
            let path = Path::new(DEFAULT_PATH);
            let mut pc = ProjectConfig::default();
            router_read_recursive(path, &mut pc, &parse).unwrap();
            println!("{pc}");
            pc._write_config_lock().unwrap();
        }
    }
}
