#[cfg(any(feature = "db-full", feature = "db-postgres-diesel"))]
pub mod diesel;

#[cfg(any(feature = "db-full", feature = "db-postgres-seaorm"))]
pub mod seaorm;

pub mod exports {
    #[cfg(any(feature = "db-full", feature = "db-postgres-diesel"))]
    pub use diesel;
    #[cfg(any(feature = "db-full", feature = "db-postgres-seaorm"))]
    pub use sea_orm;
}
