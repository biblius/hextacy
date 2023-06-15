/// Utility traits for connecting to the cache. Currently supports only redis.
#[cfg(any(feature = "db", feature = "full", feature = "redis"))]
pub mod cache;

/// Traits and macros for implementing drivers and connections.
#[cfg(any(
    feature = "db",
    feature = "full",
    feature = "postgres-diesel",
    feature = "postgres-seaorm",
    feature = "mongo"
))]
pub mod db;

/// Drivers for connecting to the database, cache and smtp.
pub mod driver;

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
pub mod web;

/// Derive macros for quick implementations of generic repository traits.
pub use hextacy_macros::contract;
