use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(author, version = "0.1", about, long_about = None)]
pub(crate) struct AlxArgs {
    #[clap(subcommand)]
    pub command: Command,
}

/// The subject of the command
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Generate a new route endpoint.
    Route(RouteCommand),
    R(RouteCommand),

    /// Generate a new middleware.
    Middleware(MiddlewareCommand),
    MW(MiddlewareCommand),
}

/// Router commands
#[derive(Debug, Args)]
pub struct RouteCommand {
    #[clap(subcommand)]
    pub command: RouteSubcommand,
}

/// Middleware commands
#[derive(Debug, Args)]
pub struct MiddlewareCommand {
    #[clap(subcommand)]
    pub command: MiddlewareSubcommand,
}

/// Commands for generating and modifying route endpoints.
#[derive(Debug, Subcommand)]
pub enum RouteSubcommand {
    /// Generate a route.
    Gen(Route),
    /// Shorthand for generate.
    G(Route),
    /// Add a contract to an existing route endpoint.
    AddContract(RouteName),
    /// Shorthand for add contract.
    AC(RouteName),
}

/// Commands for generating and modifying route endpoints.
#[derive(Debug, Subcommand)]
pub enum MiddlewareSubcommand {
    /// Generate a route.
    Gen(Middleware),
    /// Shorthand for generate.
    G(Middleware),
    /// Add a contract to an existing route endpoint.
    AddContract(MWName),
    /// Shorthand for add contract.
    AC(MWName),
}

/// Contains endpoint information
#[derive(Debug, Args)]
pub struct Route {
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
pub struct RouteName {
    /// The name of a router endpoint
    pub name: String,
}

/// Contains middleware information
#[derive(Debug, Args)]
pub struct Middleware {
    /// The name of the middleware
    pub name: String,
    /// The various services or repositories the middleware will use
    pub contracts: String,
}

#[derive(Debug, Args)]
pub struct MWName {
    /// The name of a middleware
    pub name: String,
}
