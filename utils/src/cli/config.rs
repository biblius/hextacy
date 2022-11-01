use crate::error::AlxError;
use derive_new::new;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::{fmt::Display, fs, path::Path};

const INDENT: &str = "    ";

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ProjectConfig {
    pub endpoints: Vec<Endpoint>,
    pub handlers: Vec<Handler>,
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
            writeln!(f, "- Endpoint: '{}'\nRoutes: ", ep.id)?;
            for r in &ep.routes {
                writeln!(f, "{INDENT}Method: {}\n{INDENT}Path: {}\n{INDENT}Handler: {}\n{INDENT}Service: {:?}\n{INDENT}MW: {:?}\n\n", r.method, r.path, r.handler, r.service, r.middleware)?;
            }
        }
        for h in &self.handlers {
            writeln!(f, "Handler: ")?;
            writeln!(f, "{INDENT}Name: {}", h.name)?;
            writeln!(f, "{INDENT}Inputs: {:?}", h.inputs)?;
            writeln!(f, "{INDENT}Service bound: {:?}", h.bound)?;
        }
        Ok(())
    }
}

/// Defines an endpoint in the project structure.
#[derive(Serialize, Deserialize, Debug)]
pub struct Endpoint {
    pub id: String,
    pub routes: Vec<Route>,
}

#[derive(Serialize, Deserialize, Debug, Default, new, PartialEq, Eq)]
pub struct Route {
    /// The HTTP method for the route
    pub method: String,
    /// The designated handler for the route
    pub handler: String,
    /// The path to the resource
    pub path: String,
    /// The middleware wrapped around the route, if any
    pub middleware: Option<Vec<String>>,
    pub service: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Default, new)]
pub struct Handler {
    pub name: String,
    pub inputs: Vec<HandlerInput>,
    pub bound: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, new)]
pub struct HandlerInput {
    pub ext_type: String,
    pub data_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Extractor {
    #[serde(alias = "path", alias = "Path")]
    Path,
    #[serde(alias = "query", alias = "Query")]
    Query,
    #[serde(alias = "json", alias = "Json")]
    Json,
    #[serde(alias = "form", alias = "Form")]
    Form,
    #[serde(alias = "request", alias = "Request", alias = "HttpRequest")]
    Request,
    #[serde(alias = "string", alias = "String")]
    String,
    #[serde(alias = "bytes", alias = "Bytes")]
    Bytes,
    #[serde(alias = "payload", alias = "Payload")]
    Payload,
    #[serde(alias = "data", alias = "Data")]
    Data,
    Invalid,
}

impl From<String> for Extractor {
    fn from(s: String) -> Self {
        match s.as_str() {
            "path" | "Path" => Self::Path,
            "query" | "Query" => Self::Query,
            "json" | "Json" => Self::Json,
            "form" | "Form" => Self::Form,
            "request" | "Request" | "HttpRequest" => Self::Request,
            "string" | "String" => Self::String,
            "bytes" | "Bytes" => Self::Bytes,
            "payload" | "Payload" => Self::Payload,
            "data" | "Data" => Self::Data,
            _ => Self::Invalid,
        }
    }
}
