use std::{fmt::Display, fs, path::Path};

use serde::{Deserialize, Serialize};
use serde_yaml;

use crate::error::AlxError;

const INDENT: &str = "    ";

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ProjectConfig {
    pub endpoints: Vec<Endpoint>,
    pub middleware: Option<Vec<Middleware>>,
}

impl ProjectConfig {
    pub fn _parse(yaml: String) -> Result<Self, AlxError> {
        let config = serde_yaml::from_str::<Self>(&yaml)?;
        Ok(config)
    }

    pub fn _write_config_lock(&self) -> Result<(), AlxError> {
        let config = serde_yaml::to_string(self)?;
        fs::write(Path::new("./alx_lock.yaml"), config)?;
        Ok(())
    }
}

impl Display for ProjectConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for ep in &self.endpoints {
            match ep {
                Endpoint::Scope(s) => {
                    writeln!(f, "- Scope: '{}'\n{INDENT}- Resources: ", s.name)?;
                    for r in &s.resources {
                        writeln!(
                            f,
                            "{INDENT}- Path: {}\n{INDENT}- MW: {:?}\n-{INDENT}{:#?}",
                            r.path, r.middleware, r.routes
                        )?;
                    }
                }
                Endpoint::Resource(r) => {
                    writeln!(
                        f,
                        "- Resource:\n{INDENT}- Path: {}\n{INDENT}- MW: {:?}\n",
                        r.path, r.middleware,
                    )?;
                }
            }
        }
        write!(f, "Middleware: {:?}", self.middleware,)
    }
}
/// Defines an endpoint in the project structure.
#[derive(Serialize, Deserialize, Debug)]
pub enum Endpoint {
    #[serde(alias = "scope")]
    Scope(Scope),
    #[serde(alias = "resource")]
    Resource(Resource),
}
/// Defines a group of enpoints under a shared namespace defined by `name`, with at least one resource.
/// Wrapping a scope with a `Middleware` or `Guard` will wrap all the scope's resources with the defined mw/guard.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Scope {
    pub name: String,
    pub resources: Vec<Resource>,
    pub middleware: Option<Vec<String>>,
}

/// Defines a single endpoint with at least one route.
/// Wrapping a resource with a `Middleware` or `Guard` will wrap all the resource's routes with the defined mw/guards.
#[derive(Serialize, Deserialize, Debug)]
pub struct Resource {
    pub path: String,
    pub routes: Vec<Route>,
    pub middleware: Option<Vec<String>>,
}
/// Defines the route configuration for the underlying resource. Routes can contain `Extractor`s used for extracting data
/// from the incoming http requests or the application state, and can be individually wrapped by a `Middleware` or `Guard`.
#[derive(Serialize, Deserialize, Debug)]
pub struct Route {
    /// The HTTP method for the route
    pub method: String,
    /// The path to the route handler
    pub handler: String,
    pub extractors: Option<Vec<Extractor>>,
    pub middleware: Option<Vec<String>>,
}

/// Used for registering middleware with alchemyx. A vector of `Middleware`s is defined in the root of `alx.yaml` and
/// is used to locate the middleware files when wrapping endpoints with them.
#[derive(Serialize, Deserialize, Debug)]
pub struct Middleware {
    name: String,
    path: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Extractor {
    #[serde(alias = "path")]
    Path,
    #[serde(alias = "query")]
    Query,
    #[serde(alias = "json")]
    Json,
    #[serde(alias = "form")]
    Form,
    #[serde(alias = "request")]
    Request,
    #[serde(alias = "string")]
    String,
    #[serde(alias = "bytes")]
    Bytes,
    #[serde(alias = "payload")]
    Payload,
    #[serde(alias = "state")]
    State,
}
