use async_trait::async_trait;
use thiserror::Error;

/// Used for establishing connections to a database. Implementations can be found in the `hextacy_derive`
/// crate. Manual implementations should utilise `hextacy::drivers`.
#[async_trait]
pub trait RepositoryAccess<C> {
    async fn connect(&self) -> Result<C, DatabaseError>;
}

/// Used for creating bounds on generic connections when the adapter needs to have atomic repository access.
///
/// This trait is used to normalise the API for transactions that are connection based and transactions that
/// return a transaction struct.
///
/// When transactions are connection based, the `TransactionResult` is typically
/// the connection on which the transaction is started.
///
/// When they are struct based, the adapter must implement a repository trait for both the
/// connection and transaction.
///
/// Check out the [driver module][crate::drivers::db] to see concrete implementations.
#[async_trait]
pub trait Atomic: Sized {
    type TransactionResult: Send;
    async fn start_transaction(self) -> Result<Self::TransactionResult, DatabaseError>;
    async fn commit_transaction(tx: Self::TransactionResult) -> Result<(), DatabaseError>;
    async fn abort_transaction(tx: Self::TransactionResult) -> Result<(), DatabaseError>;
}

#[macro_export]
/// Generates a struct with the given name and visibility and derives [RepositoryAccess].
///
/// Useful for reducing overall boilerplate in repository adapters.
///
/// #### 1 - Struct ident (optional)
///
/// The macro accepts an optional ident as the first parameter and will name the struct that way if provided.
///
/// #### 2 - Driver - connection, field - driver pairs
///
/// The second part of the macro uses a
///
/// `DriverIdent => ConnectionIdent,`
///
/// `field_ident => driver,`
///
/// syntax, where DriverIdent and ConnectionIdent are arbitrary driver and connection generics that can be used
/// to specify which repositories will use which drivers.
///
/// **Available drivers (for `driver`) are `diesel`, `seaorm` for postgres and `mongo`.**
///
/// #### 3 - Repository ident - Repository path
///
/// The third and final part accepts a
///
/// `RepoIdent => SomeRepository<ConnectionIdent>`
///
/// syntax, indicating which identifiers can call which repository methods.
///
/// The drivers module includes drivers which derive DBConnect for the derived
/// connections. Check out [Repository][hextacy_derive::Repository]
///
/// ### Example
///
/// ```ignore
/// adapt! {
///     Adapter, // Optional name for the generated struct    
///
///     Postgres => PgConnection, // Driver and connection
///     postgres => diesel,       // The struct field to annotate with a driver.
///
///     Mongo    => MgConnection, // Same as above, any number of pairs
///     mongo    => mongo;        // is allowed
///
///     SomeRepo => SomeRepository<Conn>, // Repository bounds
///     OtherRepo => OtherRepository<Conn>,
///     /* ... */
/// }
/// ```
///
/// This macro also provides a `new()` method whose input is anything that implements `DBConnect` for convenience.
/// `DBConnect` is automatically added to the bounds as a generic parameter for the driver.
macro_rules! adapt {
    (
        $name:ident $(in $vis:path)?,
        $(
            use $driver:ident
            for $conn_name:ident $(: $atomic_conn:path )?
            as $field:ident
            : $driver_field:ident $(,)?
        )+;
        $(
            $repository:ident < $connection:ident $(: $conn_trait:path )? > as $id:ident
        ),*
        $(,)? ;

        $($b:item)*

        ) => {
               #[allow(non_snake_case)]
               #[derive(Debug, hextacy::derive::Adapter)]
               pub $((in $vis))? struct $name<$($driver),+, $($conn_name),+, $($id),*>
               where
                  $(
                      $driver: hextacy::drivers::db::DBConnect<Connection = $conn_name> + Send + Sync,
                  )+
                   $($id: $repository <$connection> + Send + Sync),*
               {
                  $(
                      #[$driver_field($conn_name)]
                      $field: hextacy::drivers::db::Driver<$driver, $conn_name>,
                  )+
                   $($id: ::std::marker::PhantomData<$id>),*
               }

               #[allow(non_snake_case)]
               impl<$($driver),+, $($conn_name),+, $($id),*> $name <$($driver),+, $($conn_name),+, $($id),*>
               where
                  $(
                      $driver: hextacy::drivers::db::DBConnect<Connection = $conn_name> + Send + Sync,
                  )+
                   $($id: $repository <$connection> + Send + Sync),*
               {
                   pub fn new($($driver: ::std::sync::Arc<$driver>),+) -> Self {
                       Self {
                          $(
                              $field: hextacy::drivers::db::Driver::new($driver),
                          )+
                           $($id: ::std::marker::PhantomData),*
                       }
                   }
               }

               #[hextacy::component$(($vis))?]
               impl
                   <$($driver),+, $($conn_name),+, $($id),*>

                   $name

                   <$($driver),+, $($conn_name),+, $($id),*>
               where
                   Self: $(hextacy::db::RepositoryAccess<$conn_name> +)+,

                   // Apply DBConnect bounds for drivers
                   $(
                       $driver: hextacy::drivers::db::DBConnect<Connection = $conn_name> + Send + Sync,
                   )+

                   // Apply connection bounds
                   $(
                       $conn_name: $( $atomic_conn + )? Send
                   )+,

                   // Apply repository bounds
                   $(
                        $id: $repository <$connection> $(+ $repository< <$connection as $conn_trait>::TransactionResult> )? + Send + Sync
                    ),*

                    // Impl items
                   {
                       $($b)*
                   }
          };
}

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Error while attempting to establish connection: {0}")]
    Driver(#[from] super::drivers::DriverError),

    #[cfg(any(feature = "db", feature = "full", feature = "postgres-diesel"))]
    #[error("Diesel Error: {0}")]
    Diesel(#[from] diesel::result::Error),

    #[cfg(any(feature = "db", feature = "full", feature = "mongo"))]
    #[error("Mongo Error: {0}")]
    Mongo(#[from] mongodb::error::Error),

    #[cfg(any(feature = "db", feature = "full", feature = "postgres-seaorm"))]
    #[error("SeaORM Error: {0}")]
    SeaORM(#[from] sea_orm::DbErr),
}
