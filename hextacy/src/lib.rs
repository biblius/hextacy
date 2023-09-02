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
pub mod adapters;

/// Cryptographic utilities
pub mod crypto;

/// Utilities for getting and setting stuff from the env.
pub mod env;

/// A logger that can be set up to use stdout or a file.
pub mod logger;

/// Utilities for time related stuff.
pub mod time;

/// Utilities for web related stuff. Contains a WS and broker implementation
/// as well as some HTTP helpers.
#[cfg(any(feature = "full", feature = "web"))]
pub mod web;

/// Derive macro for quick implementation of component traits.
pub use hextacy_macros::{contract, Configuration};
