mod args;
use args::AlxArgs;
use clap::Parser;
use std::fmt::Write;
use std::{fs, path::Path};

use crate::args::{Command, MiddlewareSubcommand, RouteSubcommand};
pub fn main() {
    let args = AlxArgs::parse();
    println!("{:?}", args);

    match args.command {
        Command::Route(cmd) | Command::R(cmd) => match cmd.command {
            RouteSubcommand::Gen(route) | RouteSubcommand::G(route) => {
                let mut path = format!("server/src/api/router/{}", route.name);
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

                // Prepare the contracts
                let mut contents = String::new();
                for s in contracts {
                    write!(
                        contents,
                        "pub(super) trait {}{}Contract {{\n\t\n}}\n",
                        &s[..1].to_string().to_uppercase(),
                        &s[1..]
                    )
                    .unwrap();
                }
                println!("{:?}", contents);
                fs::File::create(&format!("{}/contract.rs", path)).expect("Couldn't create file");
                fs::write(&format!("{}/contract.rs", path), contents)
                    .expect("Could't write contract");

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
    }
}
