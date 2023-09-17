use serde::{Deserialize, Serialize};

/// Defines an endpoint in the project structure.
#[derive(Serialize, Deserialize, Debug)]
pub struct Endpoint {
    pub name: String,
    pub full_path: String,
    pub routes: Vec<RouteHandler>,
}

/// Love child of [Route] and [Handler]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RouteHandler {
    pub method: String,
    pub path: String,
    pub handler: Option<Handler>,
    pub middleware: Option<Vec<String>>,
    pub service: Option<String>,
    pub input: Option<Data>,
}

impl From<(&mut Route, Option<&Handler>, Option<&Data>)> for RouteHandler {
    fn from((r, h, d): (&mut Route, Option<&Handler>, Option<&Data>)) -> Self {
        Self {
            method: r.method.to_string(),
            path: r.path.to_string(),
            handler: h.cloned(),
            middleware: r.middleware.clone(),
            service: r.service.clone(),
            input: d.cloned(),
        }
    }
}

/// Intermediary struct for capturing setup functions
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct Route {
    /// The HTTP method for the route
    pub method: String,
    /// The name of the designated handler for the route
    pub handler_name: String,
    /// The path to the resource
    pub path: String,
    /// The middleware wrapped around the route, if any
    pub middleware: Option<Vec<String>>,
    /// The service this route uses
    pub service: Option<String>,
}

/// Intermediary struct for capturing all handler functions
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct Handler {
    /// Handler name
    pub name: String,
    /// The inputs (args) for this handler function
    pub inputs: Vec<HandlerInput>,
    /// Trait bounds for the handler, if any
    pub bound: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct HandlerInput {
    #[serde(rename = "extractor")]
    pub ext_type: String,
    #[serde(rename = "data")]
    pub data_type: String,
}

/// Represents a data payload expected to be received from the client
#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, Eq)]
pub struct Data {
    pub id: String,
    pub fields: Vec<Field>,
}

/// Represents a field of a client web payload (Data)
#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, Eq)]
pub struct Field {
    pub name: String,
    pub ty: String,
    pub required: bool,
    pub validation: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
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
