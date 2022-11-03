mod analyze;
mod boiler;
mod commands;
mod config;
mod error;

use crate::commands::mw::MiddlewareSubcommand;
use crate::commands::route::RouteSubcommand;
use crate::commands::Command;
use crate::config::{ConfigFormat, Endpoint, Handler, ProjectConfig, RouteHandler};
use analyze::{parse, router_read_recursive, ScanResult};
use clap::Parser;
use commands::AlxArgs;
use std::collections::HashMap;
use std::fmt::Write;
use std::{fs, path::Path};

pub const INDENT: &str = "    ";
pub const DEFAULT_PATH: &str = "server/src/api/router";
pub const FILES: [&str; 7] = [
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
            RouteSubcommand::Gen(endpoint) | RouteSubcommand::G(endpoint) => {
                let mut path = format!("{}/{}", DEFAULT_PATH, endpoint.name);

                // If a path is given switch to it
                if let Some(ref p) = endpoint.path {
                    path = format!("{}/{}", p, endpoint.name)
                }

                // Create the directory
                let router_dir = Path::new(&path);

                println!(
                    "Current directory: {:?}",
                    Path::new("./").canonicalize().unwrap()
                );

                let contracts = match endpoint.contracts {
                    Some(ref c) => c.split(',').collect::<Vec<&str>>(),
                    None => vec![],
                };

                fs::create_dir(router_dir)
                    .map_err(|e| panic!("Could not create directory at path: {}, {}", path, e))
                    .unwrap();

                for file in FILES {
                    let mut contents = String::new();
                    if file == "contract" {
                        let template =
                            fs::read_to_string("utils/src/alx/boiler/templates/__contract.rs")
                                .unwrap();
                        write!(contents, "{}", template).unwrap();
                        // Prepare the contracts if any
                        for c in &contracts {
                            write!(
                                contents,
                                "\n#[cfg_attr(test, mockall::automock)]\n#[async_trait]\npub(super) trait {}Contract {{}}\n",
                                uppercase(c)
                            )
                            .unwrap();
                        }
                    }
                    if file == "domain" {
                        // Utility closure
                        let write_bounds = |stmt: &mut String| {
                            write!(stmt, "<").unwrap();
                            for (i, c) in contracts.iter().enumerate() {
                                if i == contracts.len() - 1 {
                                    write!(stmt, "{}", &uppercase(c)[..1]).unwrap();
                                } else {
                                    write!(stmt, "{}, ", &uppercase(c)[..1]).unwrap();
                                }
                            }
                            write!(stmt, "> ").unwrap();
                        };

                        // Use statement
                        let mut use_stmt = String::from("use super::contract::");
                        if !contracts.is_empty() {
                            write!(use_stmt, "{{ServiceContract, ").unwrap();
                            for (i, c) in contracts.iter().enumerate() {
                                if i == contracts.len() - 1 {
                                    write!(use_stmt, "{}Contract", uppercase(c)).unwrap();
                                } else {
                                    write!(use_stmt, "{}Contract, ", uppercase(c)).unwrap();
                                }
                            }
                            writeln!(use_stmt, "}};").unwrap();
                        } else {
                            writeln!(use_stmt, "ServiceContract;").unwrap();
                        }

                        // Struct statement
                        let mut struct_statement =
                            format!("pub(super) struct {}", uppercase(&endpoint.name));
                        if !contracts.is_empty() {
                            write_bounds(&mut struct_statement);
                            write!(struct_statement, "\nwhere\n").unwrap();
                            for c in &contracts {
                                writeln!(
                                    struct_statement,
                                    "{INDENT}{}: {}Contract,",
                                    &uppercase(c)[..1],
                                    uppercase(c)
                                )
                                .unwrap();
                            }
                            writeln!(struct_statement, "{{").unwrap();
                            for c in &contracts {
                                writeln!(
                                    struct_statement,
                                    "{INDENT}pub {}: {},",
                                    c,
                                    &uppercase(c)[..1],
                                )
                                .unwrap();
                            }
                            writeln!(struct_statement, "}}").unwrap();
                        } else {
                            writeln!(struct_statement, ";").unwrap();
                        }

                        // Impl statement
                        let mut impl_stmt = String::from("#[async_trait]\nimpl");
                        if !contracts.is_empty() {
                            write_bounds(&mut impl_stmt);
                        }
                        write!(
                            impl_stmt,
                            "{}{}",
                            "ServiceContract for ",
                            uppercase(&endpoint.name)
                        )
                        .unwrap();
                        if !contracts.is_empty() {
                            write_bounds(&mut impl_stmt);
                            write!(impl_stmt, "\nwhere\n").unwrap();
                            for c in &contracts {
                                writeln!(
                                    impl_stmt,
                                    "{INDENT}{}: {}Contract + Send + Sync,",
                                    &uppercase(c)[..1],
                                    uppercase(c)
                                )
                                .unwrap();
                            }
                        }
                        write!(impl_stmt, "{{\n}}").unwrap();

                        writeln!(contents, "{}", use_stmt).unwrap();
                        writeln!(contents, "{}", struct_statement).unwrap();
                        writeln!(contents, "{}", impl_stmt).unwrap();
                    }
                    if file == "setup" {}
                    fs::File::create(&format!("{}/{}.rs", path, file))
                        .expect("Couldn't create file");
                    fs::write(&format!("{}/{}.rs", path, file), contents.clone())
                        .expect("Could't write to file");
                    contents.clear();
                }

                println!("{:?}", router_dir);
                println!("{:?}", endpoint);
            }
            RouteSubcommand::AddContract(route_name) | RouteSubcommand::AC(route_name) => {
                println!("{:?}", route_name)
            }
        },
        Command::Middleware(cmd) | Command::MW(cmd) => match cmd.command {
            MiddlewareSubcommand::Gen(_) | MiddlewareSubcommand::G(_) => todo!(),
            MiddlewareSubcommand::AddContract(_) | MiddlewareSubcommand::AC(_) => todo!(),
        },
        Command::Analyze(options) | Command::Anal(options) => {
            let format = match options.format {
                Some(f) => match f.as_str() {
                    "json" | "j" => ConfigFormat::Json,
                    "yaml" | "y" => ConfigFormat::Yaml,
                    _ => ConfigFormat::Both,
                },
                None => ConfigFormat::Both,
            };
            let path = Path::new(DEFAULT_PATH);
            let mut scan = ScanResult {
                handlers: HashMap::new(),
                routes: HashMap::new(),
            };
            router_read_recursive(path, &mut scan, &parse).unwrap();
            let mut pc = ProjectConfig::default();
            for ep_path in scan.routes.keys() {
                let empty = vec![];
                let handlers = match scan.handlers.get(ep_path) {
                    Some(h) => h,
                    None => &empty,
                };
                let routes = scan.routes.get(ep_path).expect("Impossible!");
                let mut ep = Endpoint {
                    id: ep_path.to_string(),
                    routes: vec![],
                };
                for route in routes {
                    let mut handler = handlers
                        .iter()
                        .filter(|h| h.name == route.handler_name)
                        .collect::<Vec<&Handler>>();
                    let handler = handler.pop();
                    let rh: RouteHandler = (route.to_owned(), handler).into();
                    ep.routes.push(rh);
                }
                pc.endpoints.push(ep);
            }
            // println!("{pc}");
            pc.write_config_lock(format).unwrap();
        }
    }
}

fn uppercase(s: &str) -> String {
    format!("{}{}", &s[..1].to_string().to_uppercase(), &s[1..])
}
