/// Core traits for implementing on data sources.
mod driver;

pub use driver::{Atomic, Driver, DriverError};

#[cfg(any(
    feature = "db-full",
    feature = "db-postgres-diesel",
    feature = "db-postgres-seaorm",
    feature = "db-mongo",
    feature = "cache-full",
    feature = "cache-redis",
    feature = "cache-inmem",
))]
/// Provides out of the box implementations for the [Driver][driver::Driver] trait.
/// Re-exports the underlying libraries used for the implementation.
pub mod adapters;

/// Cryptographic utilities
pub mod crypto;

/// Utilities for configuring stuff.
pub mod config;

/// A logger that can be set up to use stdout or a file.
// TODO: Feature flag
pub mod logger;
pub use tracing::{self, debug, error, info, warn};

/// Utilities for time related stuff.
pub mod time;

/// Utilities for web related stuff. Contains a WS and broker implementation
/// as well as some HTTP helpers.
#[cfg(any(feature = "full", feature = "web"))]
pub mod web;

/// Quality of life macros.
pub use hextacy_macros::{component, contract, Constructor, HttpResponse, State};
