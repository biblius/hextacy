use crate::{
    config::{ConfigFormat, Endpoint, Handler, HandlerInput, ProjectConfig, Route, RouteHandler},
    error::AlxError,
    DEFAULT_PATH,
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

/// Analyzes the router directory recursively and extracts routing info
pub fn handle_analyze(opts: AnalyzeOptions) {
    let format = match opts.format {
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
    router_read_recursive(path, &mut scan, &analyze).unwrap();
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

/// Recursively read the file system at the server router
pub fn router_read_recursive(
    dir: &Path,
    scan: &mut ScanResult,
    callback: &dyn Fn(&DirEntry, AlxFileType) -> Result<FileScanResult, AlxError>,
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
            router_read_recursive(&path, scan, callback)?;
        } else {
            if entry.file_name().into_string().unwrap().contains("setup") {
                println!("\n\u{1F963} Analyzing {}\n", entry.path().display());
                let setup = callback(&entry, AlxFileType::Setup).unwrap();
                if let FileScanResult::Setup(routes) = setup {
                    scan.routes.insert(dirname.clone(), routes);
                }
            }
            if entry.file_name().into_string().unwrap().contains("handler") {
                println!("\n\u{1F963} Analyzing {}\n", entry.path().display());
                let handlers = callback(&entry, AlxFileType::Handler).unwrap();
                if let FileScanResult::Handlers(h) = handlers {
                    scan.handlers.insert(dirname.clone(), h);
                }
            }
        }
    }
    Ok(())
}

/// Parse the given file according to the file type and extract routing info from it
pub fn analyze(entry: &DirEntry, file_type: AlxFileType) -> Result<FileScanResult, AlxError> {
    let mut file = File::open(entry.path())?;
    let mut src = String::new();
    file.read_to_string(&mut src).expect("Unable to read file");
    let syntax = syn::parse_file(&src).expect("Unable to parse file");
    // Grab the endpoint name
    let ep_name = entry.path();
    let ep_name = ep_name
        .as_os_str()
        .to_str()
        .unwrap()
        .split('/')
        .collect::<Vec<&str>>();
    let ep_name = ep_name[ep_name.len() - 2];
    println!("Scanning endpoint directory: {}", ep_name);
    match file_type {
        /*
         * Setup -- Doesn't work with scopes currently
         */
        AlxFileType::Setup => {
            // Extract the functions. Only the `routes()` function
            // should be top level in this file.
            let routes_fn = syntax
                .items
                .into_iter()
                .filter_map(|e: syn::Item| match e {
                    syn::Item::Fn(f) => Some(f),
                    _ => None,
                })
                .collect::<Vec<syn::ItemFn>>();

            // Get all the statements from its block. This will include all
            // the service initializations and cfg.service() calls
            let mut routes_fn_inner = match routes_fn.first() {
                Some(calls) => calls.block.stmts.clone(),
                None => vec![],
            };

            // Filter out the cfg.service calls
            let inner_calls = routes_fn_inner
                .drain(..)
                .filter(|stmt| matches!(stmt, syn::Stmt::Semi(_, _)))
                .collect::<Vec<syn::Stmt>>();

            println!("Found {} inner cfg.service calls", inner_calls.len());

            let mut setup = Vec::<Route>::new();
            let mut route = Route::default();

            for call in inner_calls {
                // cfg.service() calls will always be method calls
                if let syn::Stmt::Semi(syn::Expr::MethodCall(cfg_service_call), _) = call {
                    // The target should always be either the cfg or a *scope -- *todo
                    if let syn::Expr::Path(ref service_path) = *cfg_service_call.receiver {
                        let target = &service_path
                            .path
                            .segments
                            .first()
                            .unwrap()
                            .ident
                            .to_string();

                        if target == "cfg" && cfg_service_call.method == "service" {
                            // These calls always have one argument
                            let arg = cfg_service_call.args.first().unwrap();

                            // And it's probably going to have nested route calls
                            // inside, so we check for those. If the need arises we'll
                            // check for regular expr calls and expr path as well.
                            if let syn::Expr::MethodCall(ref service_call) = arg {
                                let mut level = 0;
                                let mut route_config = HashMap::<usize, Vec<String>>::new();
                                scan_setup(
                                    service_call.clone(),
                                    &mut route,
                                    &mut level,
                                    &mut route_config,
                                );
                                println!("Mapped: {:?}", route_config);
                            }
                            // println!("ARG: {:#?}", arg);
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

/// Scan a setup.rs file for route info
fn scan_setup(
    expr_meth_call: ExprMethodCall,
    route: &mut Route,
    level: &mut usize,
    stuff: &mut HashMap<usize, Vec<String>>,
) {
    // If the receiver is another method call, scan it recursively.
    if let syn::Expr::MethodCall(ref meth_call) = *expr_meth_call.receiver {
        scan_setup(meth_call.clone(), route, level, stuff);
    }

    // This checks for the resource("/path") string literal.
    if let syn::Expr::Call(ref call) = *expr_meth_call.receiver {
        if let Some(syn::Expr::Lit(ref p)) = call.args.first() {
            if let syn::Lit::Str(ref path) = p.lit {
                route.path = path.value();
                stuff
                    .entry(*level)
                    .and_modify(|e| e.push(path.value()))
                    .or_insert_with(|| vec![path.value()]);
            }
        }
    }

    // Iterate through all the method call arguments
    for mut arg in expr_meth_call.args {
        // Middleware wrappers, i.e. some_guard in `.wrap(some_guard)` will be a path argument
        if let syn::Expr::Path(ref path) = arg {
            if let Some(wrapper) = path.path.get_ident() {
                if let Some(ref mut mw) = route.middleware {
                    mw.push(wrapper.to_string())
                } else {
                    route.middleware = Some(vec![wrapper.to_string()])
                }
                stuff
                    .entry(*level)
                    .and_modify(|e| e.push(wrapper.to_string()))
                    .or_insert_with(|| vec![wrapper.to_string()]);
            }
        }

        // Check for more method calls
        if let syn::Expr::MethodCall(ref mut meth_call) = arg {
            // And if the receiver is another one scan recursively
            if let syn::Expr::MethodCall(ref call) = *meth_call.receiver {
                scan_setup(call.clone(), route, level, stuff);
            }

            // Otherwise check if the receiver is a function call
            if let syn::Expr::Call(ref mut call) = *meth_call.receiver {
                // Look for a web::method() call
                if let syn::Expr::Path(ref mut call) = *call.func {
                    let methods = &mut call.path.segments;
                    route.method = methods.pop().unwrap().value().ident.to_string();

                    stuff
                        .entry(*level)
                        .and_modify(|e| e.push(route.method.clone()))
                        .or_insert_with(|| vec![route.method.clone()]);
                }
                // Look for a path literal i.e. web::resource("/something")
                if let Some(syn::Expr::Lit(ref p)) = call.args.first() {
                    if let syn::Lit::Str(ref path) = p.lit {
                        route.path = path.value();
                        stuff
                            .entry(*level)
                            .and_modify(|e| e.push(path.value()))
                            .or_insert_with(|| vec![path.value()]);
                    }
                }
            }

            // We also have to check for wrappers in method call arguments
            if let syn::Expr::Path(ref path) = *meth_call.receiver {
                if let Some(wrapper) = path.path.get_ident() {
                    if let Some(ref mut mw) = route.middleware {
                        mw.push(wrapper.to_string())
                    } else {
                        route.middleware = Some(vec![wrapper.to_string()])
                    }
                    stuff
                        .entry(*level)
                        .and_modify(|e| e.push(wrapper.to_string()))
                        .or_insert_with(|| vec![wrapper.to_string()]);
                }
            }

            // Get the name of the handler
            if let Some(syn::Expr::Path(route_path)) = meth_call.args.first() {
                let mut service = None;
                let mut handlers = route_path
                    .path
                    .segments
                    .iter()
                    .filter_map(|p| {
                        if p.ident != "handler" {
                            // Get the service associated with the handler if any
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

                let h = handlers.pop().unwrap();
                let s = service.clone().unwrap_or_default();
                // Insert stuff into the map
                stuff
                    .entry(*level)
                    .and_modify(|e| e.push(s.clone()))
                    .or_insert_with(|| vec![s.clone()]);
                stuff
                    .entry(*level)
                    .and_modify(|e| e.push(h.to_string()))
                    .or_insert_with(|| vec![h.to_string()]);
                route.handler_name = h;
                route.service = service;
            }
        }
    }
    *level += 1;
}

/// Scan the handler.rs file for handler info
fn scan_handlers(functions: Vec<syn::ItemFn>) -> Vec<Handler> {
    let mut handlers = Vec::<Handler>::new();

    for hand in functions {
        // Grab the name of the handler
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
