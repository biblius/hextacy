pub(super) mod scanners;
pub(super) mod util;

use crate::{
    analyzer::scanners::{scan_data, scan_handlers},
    config::{ConfigFormat, Data, Endpoint, Handler, ProjectConfig, Route, RouteHandler},
    error::AlxError,
    DEFAULT_PATH,
};
use clap::Args;
use scanners::scan_setup;
use std::{
    collections::HashMap,
    fs::{self, DirEntry},
    path::Path,
};

#[derive(Debug)]
pub struct ScanResult {
    pub handlers: HashMap<String, Vec<Handler>>,
    pub routes: HashMap<String, Vec<Route>>,
    pub data: HashMap<String, Vec<Data>>,
}

pub enum FileScanResult {
    Handlers(Vec<Handler>),
    Routes(Vec<Route>),
    Data(Vec<Data>),
}

pub enum AlxFileType {
    Setup,
    Handler,
    Data,
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
        data: HashMap::new(),
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
/// The `callback()` is a function to execute once we find an entry we're interested in,
/// which in our case is `analyze`
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
            println!("\n\u{1F963} Analyzing {}\n", entry.path().display());
            if entry.file_name().into_string().unwrap().contains("setup") {
                let routes = callback(&entry, AlxFileType::Setup).unwrap();
                if let FileScanResult::Routes(routes) = routes {
                    scan.routes.insert(dirname.clone(), routes);
                }
            }
            if entry.file_name().into_string().unwrap().contains("handler") {
                let handlers = callback(&entry, AlxFileType::Handler).unwrap();
                if let FileScanResult::Handlers(handlers) = handlers {
                    scan.handlers.insert(dirname.clone(), handlers);
                }
            }
            if entry.file_name().into_string().unwrap().contains("data") {
                let data = callback(&entry, AlxFileType::Data).unwrap();
                if let FileScanResult::Data(data) = data {
                    scan.data.insert(dirname.clone(), data);
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
