use clap::{Args, Subcommand};

/// Router commands
#[derive(Debug, Args)]
pub(crate) struct RouteCommand {
    #[clap(subcommand)]
    pub command: RouteSubcommand,
}
/// Commands for generating and modifying route endpoints.
#[derive(Debug, Subcommand)]
pub(crate) enum RouteSubcommand {
    /// Generate a route.
    Gen(Route),
    /// Shorthand for generate.
    G(Route),
    /// Add a contract to an existing route endpoint.
    AddContract(RouteName),
    /// Shorthand for add contract.
    AC(RouteName),
}

/// Contains endpoint information
#[derive(Debug, Args)]
pub(crate) struct Route {
    /// The name of the router endpoint.
    pub name: String,
    #[arg(short, long)]
    /// The various services or repositories the endpoint will use. Comma seperated.
    pub contracts: Option<String>,
    #[arg(short, long)]
    /// The path to the API you wish to generate this endpoint. Defaults to ./server/api/router
    pub path: Option<String>,
}

#[derive(Debug, Args)]
pub(crate) struct RouteName {
    /// The name of a router endpoint
    pub name: String,
}
