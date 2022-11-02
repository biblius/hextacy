use crate::{
    config::{Handler, HandlerInput, Route},
    error::AlxError,
};
use clap::Args;
use colored::Colorize;
use std::{
    collections::HashMap,
    fs::{self, DirEntry, File},
    io::Read,
    path::Path,
};
use syn::ExprMethodCall;

type EndpointID = String;

#[derive(Debug)]
pub struct ScanResult {
    pub handlers: HashMap<EndpointID, Vec<Handler>>,
    pub routes: HashMap<EndpointID, Vec<Route>>,
}

pub enum FileScanResult {
    Handlers(Vec<Handler>),
    Setup(Vec<Route>),
}

pub enum AlxFileType {
    Setup,
    Handler,
}

#[derive(Debug, Args)]
pub struct AnalyzeOptions {
    /// Accepted values are "json" | "j" for JSON, "yaml" | "y" for Yaml.
    /// Creates both by default.
    #[arg(short, long)]
    pub format: Option<String>,
}

/// Recursively read the file system at the specified path.
pub fn router_read_recursive(
    dir: &Path,
    scan: &mut ScanResult,
    cb: &dyn Fn(&DirEntry, AlxFileType) -> Result<FileScanResult, AlxError>,
) -> Result<(), AlxError> {
    println!(
        "\n\u{1F4D6} Reading {} \u{1F4D6}",
        dir.to_str().expect("Couldn't read directory name")
    );
    for entry in fs::read_dir(dir)? {
        let entry = entry?;

        let dirname = dir.to_string_lossy().to_string();

        let path = entry.path();
        if path.is_dir() {
            router_read_recursive(&path, scan, cb)?;
        } else {
            if entry.file_name().into_string().unwrap().contains("setup") {
                println!("\n\u{1F963} Analyzing {}\n", entry.path().display());
                let setup = cb(&entry, AlxFileType::Setup).unwrap();
                if let FileScanResult::Setup(routes) = setup {
                    scan.routes.insert(dirname.clone(), routes);
                }
            }
            if entry.file_name().into_string().unwrap().contains("handler") {
                println!("\n\u{1F963} Analyzing {}\n", entry.path().display());
                let handlers = cb(&entry, AlxFileType::Handler).unwrap();
                if let FileScanResult::Handlers(h) = handlers {
                    scan.handlers.insert(dirname.clone(), h);
                }
            }
        }
    }
    Ok(())
}

pub fn parse(entry: &DirEntry, file_type: AlxFileType) -> Result<FileScanResult, AlxError> {
    let mut file = File::open(entry.path())?;
    let mut src = String::new();
    file.read_to_string(&mut src).expect("Unable to read file");
    let syntax = syn::parse_file(&src).expect("Unable to parse file");
    // Grab the endpoint name
    let filename = entry.path();
    let filename = filename
        .as_os_str()
        .to_str()
        .unwrap()
        .split('/')
        .collect::<Vec<&str>>();
    let filename = filename[filename.len() - 2];
    println!("Scanning endpoint directory: {}", filename);
    match file_type {
        /*
         * Setup -- Doesn't work with scopes currently
         */
        AlxFileType::Setup => {
            // Extract the functions
            let routes_fn = syntax
                .items
                .into_iter()
                .filter_map(|e: syn::Item| match e {
                    syn::Item::Fn(f) => Some(f),
                    _ => None,
                })
                .collect::<Vec<syn::ItemFn>>();

            // Get all the config calls
            let mut function_inner = match routes_fn.first() {
                Some(calls) => calls.block.stmts.clone(),
                None => vec![],
            };

            let config_calls = function_inner
                .drain(..)
                .filter(|stmt| matches!(stmt, syn::Stmt::Semi(_, _)))
                .collect::<Vec<syn::Stmt>>();
            println!("Found {} cfg calls", config_calls.len());

            let mut setup = Vec::<Route>::new();
            let mut route = Route::default();

            // We want only the method calls i.e. cfg.service()
            for call in config_calls {
                if let syn::Stmt::Semi(syn::Expr::MethodCall(service_call), _) = call {
                    if let syn::Expr::Path(ref service_path) = *service_call.receiver {
                        let target = &service_path
                            .path
                            .segments
                            .first()
                            .unwrap()
                            .ident
                            .to_string();

                        if target == "cfg" && service_call.method == "service" {
                            for arg in service_call.args {
                                // println!("ARG: {:?}", arg);
                                if let syn::Expr::MethodCall(ref resource_call) = arg {
                                    scan_setup(resource_call.clone(), &mut route);
                                }
                            }
                        }
                    }
                }
                if route != Route::default() {
                    setup.push(route);
                    route = Route::default();
                }
            }
            Ok(FileScanResult::Setup(setup))
        }
        /*
         * Handler
         */
        AlxFileType::Handler => {
            // Grab all the functions from the file
            let functions = syntax
                .items
                .into_iter()
                .filter_map(|e: syn::Item| match e {
                    syn::Item::Fn(f) => Some(f),
                    _ => None,
                })
                .collect::<Vec<syn::ItemFn>>();
            Ok(FileScanResult::Handlers(scan_handlers(functions)))
        }
    }
}

fn scan_setup(p: ExprMethodCall, route: &mut Route) {
    if let syn::Expr::MethodCall(ref mc) = *p.receiver {
        scan_setup(mc.clone(), route);
    }
    if let syn::Expr::Call(ref c) = *p.receiver {
        if let Some(syn::Expr::Lit(ref p)) = c.args.first() {
            if let syn::Lit::Str(ref path) = p.lit {
                route.path = path.value();
            }
        }
    }
    for arg in p.args {
        if let syn::Expr::Path(ref p) = arg {
            if let Some(wrapper) = p.path.get_ident() {
                if let Some(ref mut mw) = route.middleware {
                    mw.push(wrapper.to_string())
                } else {
                    route.middleware = Some(vec![wrapper.to_string()])
                }
            }
        }
        // Check if the arg is a method call
        if let syn::Expr::MethodCall(ref mc) = arg {
            // Recursively check nested calls
            if let syn::Expr::MethodCall(ref m_call) = *mc.receiver {
                scan_setup(m_call.clone(), route);
            }
            // Get the route
            if let syn::Expr::Call(ref m_call) = *mc.receiver {
                if let syn::Expr::Path(ref call) = *m_call.func {
                    let mut methods = call
                        .path
                        .segments
                        .iter()
                        .filter_map(|p| {
                            if p.ident != "web" {
                                return Some(p.ident.to_string());
                            }
                            None
                        })
                        .collect::<Vec<String>>();
                    route.method = methods.pop().unwrap();
                }
                if let Some(syn::Expr::Lit(ref p)) = m_call.args.first() {
                    if let syn::Lit::Str(ref path) = p.lit {
                        route.path = path.value();
                    }
                }
            }
            // Check if we have a receiving wrapper
            if let syn::Expr::Path(ref m_call) = *mc.receiver {
                if let Some(wrapper) = m_call.path.get_ident() {
                    if let Some(ref mut mw) = route.middleware {
                        mw.push(wrapper.to_string())
                    } else {
                        route.middleware = Some(vec![wrapper.to_string()])
                    }
                }
            }
            // Get the name of the handler
            if let Some(syn::Expr::Path(route_path)) = mc.args.first() {
                let mut service = None;
                let mut handlers = route_path
                    .path
                    .segments
                    .iter()
                    .filter_map(|p| {
                        if p.ident != "handler" {
                            // Get the service associated with the handler
                            if let syn::PathArguments::AngleBracketed(ref args) = p.arguments {
                                for arg in &args.args {
                                    if let syn::GenericArgument::Type(syn::Type::Path(p)) = arg {
                                        service = Some(
                                            p.path.segments.first().unwrap().ident.to_string(),
                                        );
                                    }
                                }
                            }
                            return Some(p.ident.to_string());
                        }
                        None
                    })
                    .collect::<Vec<String>>();
                route.handler_name = handlers.pop().expect("Route must have handler");
                route.service = service;
            }
        }
    }
}

fn scan_handlers(functions: Vec<syn::ItemFn>) -> Vec<Handler> {
    let mut handlers = Vec::<Handler>::new();

    for hand in functions {
        // Grab the name of the handler and init it
        let name = hand.sig.ident.to_string();

        // Check if it has any bounds
        let bound = match hand.sig.generics.params.first() {
            Some(param) => match param {
                syn::GenericParam::Type(ty) => {
                    let mut typ = ty.ident.to_string();
                    if let Some(bound) = ty.bounds.first() {
                        match bound {
                            syn::TypeParamBound::Trait(tb) => {
                                typ = format!(
                                    "{}: {}",
                                    typ,
                                    tb.path.segments.first().unwrap().ident,
                                );
                                Some(typ)
                            }
                            syn::TypeParamBound::Lifetime(_) => todo!(),
                        }
                    } else {
                        Some(typ)
                    }
                }
                syn::GenericParam::Lifetime(_) => todo!(),
                syn::GenericParam::Const(_) => todo!(),
            },
            None => None,
        };

        let mut handler = Handler {
            name,
            inputs: vec![],
            bound,
        };

        hand.sig.inputs.into_iter().for_each(|fn_arg| match fn_arg {
            // Skip references to self in handlers
            syn::FnArg::Receiver(f) => {
                println!("{} {:?}", "Found self in handler?".red(), f)
            }
            // And iterate through all the args of the function
            syn::FnArg::Typed(args) => {
                if let syn::Type::Path(p) = *args.ty {
                    // Iterate through all the type params
                    for seg in p.path.segments {
                        // We don't care about the web prefix
                        if seg.ident != "web" {
                            // The identity is the extractor type which holds the data type,
                            // mostly in angle bracket argument form
                            let ext_type = seg.ident.to_string();
                            let data_type = match seg.arguments {
                                syn::PathArguments::AngleBracketed(arg) => {
                                    // There's usually just one angle bracketed arg since all the data
                                    // should come from some kind of wrapper struct from data.rs
                                    match arg.args.first().unwrap() {
                                        syn::GenericArgument::Type(t) => {
                                            // The same goes for the path, this is where we'll find the
                                            // data type
                                            if let syn::Type::Path(t) = t {
                                                t.path.segments.first().unwrap().ident.to_string()
                                            } else {
                                                panic!("Found a funky syn Type")
                                            }
                                        }
                                        _ => panic!("Found some funky angle bracket arguments"),
                                    }
                                }
                                syn::PathArguments::Parenthesized(_) => String::new(),
                                syn::PathArguments::None => String::new(),
                            };
                            handler.inputs.push(HandlerInput {
                                ext_type,
                                data_type,
                            })
                        }
                    }
                }
            }
        });
        println!("Created {:?}", handler);
        handlers.push(handler);
    }
    handlers
}
