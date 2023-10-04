/// Core traits for implementing on data sources.
mod driver;

pub use driver::{Atomic, Driver, DriverError};

/// Provides out of the box implementations for the [Driver][driver::Driver] trait.
/// Re-exports the underlying libraries used for the implementation.
#[cfg(any(
    feature = "db-postgres-diesel",
    feature = "db-postgres-seaorm",
    feature = "db-mysql-diesel",
    feature = "db-mysql-seaorm",
    feature = "db-sqlite-diesel",
    feature = "db-sqlite-seaorm",
    feature = "db-mongo",
    feature = "cache-redis",
    feature = "cache-inmem",
    feature = "email"
))]
pub mod adapters;

pub mod queue;

#[cfg(feature = "crypto")]
/// Cryptographic utilities
pub mod crypto;

/// Utilities for loading dotenv and grabbing stuff from the env.
pub mod env;

/// A logger that can be set up to use stdout or a file.
pub mod logger;

/// Utilities for time related stuff.
pub mod time;

/// Utilities for web related stuff. Contains a WS and broker implementation
/// as well as some HTTP helpers.
#[cfg(feature = "web")]
pub mod web;

#[cfg(feature = "web")]
pub use hextacy_macros::RestResponse;

/// Quality of life macros.
pub use hextacy_macros::{component, contract, Constructor, State};

/// Re-exported libraries for convenience and out of the box implementations.
/// This will vary based on features flags.
///
/// See [adapters][crate::adapters] for concrete driver implementations.
pub mod exports {
    #[cfg(feature = "cache-redis")]
    pub use deadpool_redis;
    #[cfg(any(
        feature = "db-postgres-diesel",
        feature = "db-mysql-diesel",
        feature = "db-sqlite-diesel"
    ))]
    pub use diesel;
    #[cfg(feature = "email")]
    pub use lettre;
    #[cfg(feature = "db-mongo")]
    pub use mongodb;
    #[cfg(any(
        feature = "db-postgres-seaorm",
        feature = "db-mysql-seaorm",
        feature = "db-sqlite-seaorm"
    ))]
    pub use sea_orm;

    #[cfg(feature = "crypto")]
    pub use {bcrypt, hmac, jsonwebtoken, rand, rsa, sha2, thotp, uuid};
}

/// A trait for hooking services up to application configurations. The usual application is simply
/// instantiating a service and calling a framework specific function to hook it up to a service.
pub trait Configure<State, Config> {
    fn configure(state: &State, cfg: &mut Config);
}
