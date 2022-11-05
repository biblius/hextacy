pub(crate) mod mw;
pub(crate) mod route;

use self::{mw::MiddlewareCommand, route::RouteCommand};
use crate::analyzer::AnalyzeOptions;
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(author, version = "0.1", about, long_about = None)]
pub(crate) struct AlxArgs {
    #[clap(subcommand)]
    pub command: Command,
}

/// The top level command
#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    /// Generate a new route endpoint.
    Route(RouteCommand),
    R(RouteCommand),

    /// Generate a new middleware.
    Middleware(MiddlewareCommand),
    MW(MiddlewareCommand),

    /// Analyze the router directory and generate an alx.yaml file
    Analyze(AnalyzeOptions),
    Anal(AnalyzeOptions),
}
