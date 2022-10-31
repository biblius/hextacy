use clap::{Args, Subcommand};

/// Middleware commands
#[derive(Debug, Args)]
pub(crate) struct MiddlewareCommand {
    #[clap(subcommand)]
    pub command: MiddlewareSubcommand,
}

/// Commands for generating and modifying route endpoints.
#[derive(Debug, Subcommand)]
pub(crate) enum MiddlewareSubcommand {
    /// Generate a route.
    Gen(Middleware),
    /// Shorthand for generate.
    G(Middleware),
    /// Add a contract to an existing route endpoint.
    AddContract(MWName),
    /// Shorthand for add contract.
    AC(MWName),
}

/// Contains middleware information
#[derive(Debug, Args)]
pub(crate) struct Middleware {
    /// The name of the middleware
    pub name: String,
    /// The various services or repositories the middleware will use
    pub contracts: String,
}

#[derive(Debug, Args)]
pub(crate) struct MWName {
    /// The name of a middleware
    pub name: String,
}
