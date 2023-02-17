use super::scanners::scan_setup;
use crate::{
    analyzer::scanners::{scan_data, scan_handlers},
    config::{ConfigFormat, Data, Endpoint, Handler, ProjectConfig, Route, RouteHandler},
    error::AlxError,
    print,
};
use clap::Args;
use std::{
    collections::HashMap,
    fs::{self, DirEntry},
    path::Path,
};

/// Maps endpoint names to their respective properties.
///
/// Scanned files include:
///
/// - handler.rs
/// - setup.rs
/// - data.rs
#[derive(Debug)]
pub struct ScanResult {
    pub handlers: HashMap<String, Vec<Handler>>,
    pub routes: HashMap<String, Vec<Route>>,
    pub data: HashMap<String, Vec<Data>>,
}

/// Enumeration of the types of files
#[derive(Debug)]
pub enum FileScanResult {
    Handlers(Vec<Handler>),
    Routes(Vec<Route>),
    Data(Vec<Data>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlxFileType {
    Setup,
    Handler,
    Data,
}

/// Analyze the router directory and generate an alx.yaml/json file
#[derive(Debug, Args)]
pub struct AnalyzeOptions {
    /// Accepted values are "json" | "j" for JSON, "yaml" | "y" for Yaml.
    /// Creates both by default.
    #[arg(short, long)]
    pub format: Option<String>,
    /// Print what's going on to std out
    #[arg(short, long, action)]
    pub verbose: bool,
    /// Specify the path to read from.
    #[arg(short, long)]
    pub path: Option<String>,
}

/// Analyzes the router directory recursively and extracts routing info. Assembles the ProjectConfig struct
/// after it calling the scanners to do their thing.
pub fn handle(opts: AnalyzeOptions, api_path: &str) {
    let format = match opts.format {
        Some(f) => match f.as_str() {
            "json" | "j" => ConfigFormat::Json,
            "yaml" | "y" => ConfigFormat::Yaml,
            _ => ConfigFormat::Both,
        },
        None => ConfigFormat::Both,
    };

    let mut scan = ScanResult {
        handlers: HashMap::new(),
        routes: HashMap::new(),
        data: HashMap::new(),
    };

    let path = format!("{api_path}/router");

    router_read_recursive(Path::new(&path), &mut scan, &analyze, None).unwrap();

    // println!("MY CURRENT CONFIG {:?}", scan);

    let mut pc = ProjectConfig::default();

    for ep_name in scan.routes.keys() {
        // Grab the endpoint name
        let file_path = format!("{path}/{ep_name}");
        // Get the handlers under the current path
        let empty = vec![];
        let handlers = match scan.handlers.get(ep_name) {
            Some(h) => h,
            None => &empty,
        };

        // Get the data
        let empty = vec![];
        let data = match scan.data.get(ep_name) {
            Some(h) => h,
            None => &empty,
        };

        // Get the routes
        let routes = scan.routes.get(ep_name).expect("Impossible!");
        let mut ep = Endpoint {
            name: ep_name.to_string(),
            full_path: file_path.to_string(),
            routes: vec![],
        };

        for route in routes {
            // Get the handler associated with the route name
            let mut handler = handlers
                .iter()
                .filter(|h| h.name == route.handler_name)
                .collect::<Vec<&Handler>>();

            // Get the data associated with the handler name
            let mut data = data
                .iter()
                .filter(|d| {
                    for h in handler.iter() {
                        for i in h.inputs.iter() {
                            if i.data_type == d.id {
                                return true;
                            }
                        }
                    }
                    false
                })
                .collect::<Vec<&Data>>();

            let handler = handler.pop();
            let data = data.pop();

            let rh = RouteHandler {
                method: route.method.clone(),
                path: route.path.clone(),
                handler: handler.cloned(),
                middleware: route.middleware.clone(),
                service: route.service.clone(),
                input: data.cloned(),
            };
            ep.routes.push(rh);
        }
        pc.endpoints.push(ep);
    }
    println!("Writing alx_lock{format}");
    pc.write_config_lock(format).unwrap();
}

/// Recursively read the file system at the server router
/// The `callback()` is a function to execute once we find an entry we're interested in,
/// which in our case is [analyze]
pub fn router_read_recursive(
    dir: &Path,
    scan: &mut ScanResult,
    callback: &dyn Fn(&DirEntry, AlxFileType) -> Result<FileScanResult, AlxError>,
    dir_type: Option<AlxFileType>,
) -> Result<(), AlxError> {
    print(&format!(
        "\u{1F4D6} Reading {} \u{1F4D6}",
        dir.to_str().expect("Couldn't read directory name")
    ));

    for entry in fs::read_dir(dir)? {
        let entry = entry?;

        // Ep name is the id of the endpoint, i.e. router/<id>
        let ep_name = if dir_type.is_some() {
            let name = dir.to_string_lossy().to_string();
            let name = name.split('/').collect::<Vec<&str>>();
            name[name.len() - 2].to_string()
        } else {
            let name = dir.to_string_lossy().to_string();
            let n = name.split('/').collect::<Vec<&str>>();
            n[n.len() - 1].to_string()
        };

        let path = entry.path();

        // Check if it's a directory and run according to the predefined ones
        if path.is_dir() {
            if path.ends_with("handler") {
                router_read_recursive(&path, scan, callback, Some(AlxFileType::Handler))?;
            } else if path.ends_with("setup") {
                router_read_recursive(&path, scan, callback, Some(AlxFileType::Setup))?;
            } else if path.ends_with("data") {
                router_read_recursive(&path, scan, callback, Some(AlxFileType::Data))?;
            } else {
                router_read_recursive(&path, scan, callback, None)?;
            }
        } else {
            let file_name = entry.file_name().into_string().unwrap();

            print(&format!("\u{1F440} Analyzing {}", entry.path().display()));

            if let Some(dir_type) = dir_type.clone() {
                match callback(&entry, dir_type)? {
                    FileScanResult::Handlers(ref mut handlers) => {
                        scan.handlers
                            .entry(ep_name)
                            .and_modify(|entry| entry.append(handlers))
                            .or_insert_with(|| handlers.to_vec());
                    }
                    FileScanResult::Routes(ref mut routes) => {
                        scan.routes
                            .entry(ep_name)
                            .and_modify(|entry| entry.append(routes))
                            .or_insert_with(|| routes.to_vec());
                    }
                    FileScanResult::Data(data) => {
                        scan.data.insert(ep_name.to_string(), data);
                    }
                }
                continue;
            }

            let in_setup = dir_type.is_some() && dir_type.unwrap() == AlxFileType::Setup;

            if file_name.contains("setup") || in_setup {
                let routes = callback(&entry, AlxFileType::Setup)?;
                if let FileScanResult::Routes(routes) = routes {
                    scan.routes.insert(ep_name.to_string(), routes);
                }
            }
            if file_name.contains("handler") {
                let handlers = callback(&entry, AlxFileType::Handler)?;
                if let FileScanResult::Handlers(handlers) = handlers {
                    scan.handlers.insert(ep_name.to_string(), handlers);
                }
            }
            if file_name.contains("data") {
                let data = callback(&entry, AlxFileType::Data)?;
                if let FileScanResult::Data(data) = data {
                    scan.data.insert(ep_name.to_string(), data);
                }
            }
        }
    }
    Ok(())
}

/// Parse the given file according to the file type and extract routing info from it
pub fn analyze(entry: &DirEntry, file_type: AlxFileType) -> Result<FileScanResult, AlxError> {
    // Get the file syntax struct
    let src = fs::read_to_string(entry.path())?;
    let syntax = syn::parse_file(&src).expect("Unable to parse file");
    match file_type {
        AlxFileType::Setup => {
            // Extract the functions. Only the `routes()` function
            // should be top level in this file and this vec should
            // in theory only contain that one
            let functions = syntax
                .items
                .into_iter()
                .filter_map(|e: syn::Item| match e {
                    syn::Item::Fn(f) => Some(f),
                    _ => None,
                })
                .collect::<Vec<syn::ItemFn>>();

            let routes = scan_setup(functions);
            Ok(FileScanResult::Routes(routes))
        }
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
            let handlers = scan_handlers(functions);
            Ok(FileScanResult::Handlers(handlers))
        }
        AlxFileType::Data => {
            // Filter out only the structs
            let data = syntax
                .items
                .into_iter()
                .filter(|item| {
                    if let syn::Item::Struct(_) = item {
                        return true;
                    }
                    false
                })
                .collect::<Vec<syn::Item>>();
            let data = scan_data(data);
            Ok(FileScanResult::Data(data))
        }
    }
}
