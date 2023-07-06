use async_trait::async_trait;

use crate::driver::DriverError;

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

    async fn start_transaction(self) -> Result<Self::TransactionResult, DriverError>;
    async fn commit_transaction(tx: Self::TransactionResult) -> Result<(), DriverError>;
    async fn abort_transaction(tx: Self::TransactionResult) -> Result<(), DriverError>;
}

#[macro_export]
/// Generates a struct with the given name and visibility.
/// Intended to be used for service components accessing the database or cache.
///
/// ### Example
///
/// ```ignore
/// drive! {
///     Adapter in super, // Name of the generated struct, optional visibility
///
///     // This line adds the generics `D` and `Connection` to the struct
///     // as well as a field named `driver` which gets used in the underlying
///     // `RepositoryAccess` implementation. Specifically, the field will be
///     // a `Driver<D, Connection>`.
///     use D for Connection as driver;
///
///     // This adds another generic parameter `User` for the struct and
///     // will bind it to a `UserRepository<Connection>`. This binds
///     // the user repository to use the connection from the previous line.
///     // If multiple use clauses are given, any number repository-connection
///     // combinations are possible, so long the necessary adapters exist.
///     User: UserRepository<Connection>,
/// }
/// ```
///
/// The main use case is to consisely create an adapter followed by an impl block annotated with `#[component]`,
/// specifying an adapters interaction with the database.
///
/// The impl block, for example for some kind of user service would then have the form of:
///
/// ```ignore
///
///  #[component]
///  impl<D, C, User, Session, OAuth> RepositoryComponent<D, C, User, Session, OAuth>
///  where
///      C: Atomic + Send,
///      D: Connect<Connection = C> + Send + Sync,
///      User: UserRepository<C> + UserRepository<<C as Atomic>::TransactionResult> + Send + Sync,
/// ```
///
/// The `Atomic` bound is optional and is solely here to demonstrate what an ACID compliant implementation
/// would look like.
///
/// This macro also provides a `new()` method whose input is anything that implements `Connect` for convenience.
macro_rules! drive {
    (
        $name:ident $(in $vis:path)?,
        $(
            use $driver:ident
            for $conn_name:ident
            as $field:ident
            $(in $field_vis:path)? $(,)?
        )+;
        $(
            $id:ident : $repository:ident < $connection:ident >
        ),*
        $(,)?
    ) => {
            #[allow(non_snake_case)]
            #[derive(Debug)]
            pub $((in $vis))? struct $name<$($driver),+, $($conn_name),+, $($id),*>
            where
               $(
                   $driver: hextacy::driver::Driver<Connection = $conn_name> + Send + Sync,
               )+
                $($id: $repository <$connection> + Send + Sync),*
            {
               $(
                 $( pub (in $field_vis) )? $field: ::std::sync::Arc<$driver>,
               )+
                $($id: ::std::marker::PhantomData<$id>),*
            }

            #[allow(non_snake_case)]
            impl<$($driver),+, $($conn_name),+, $($id),*> $name <$($driver),+, $($conn_name),+, $($id),*>
            where
               $(
                   $driver: hextacy::driver::Driver<Connection = $conn_name> + Send + Sync,
               )+
                $($id: $repository <$connection> + Send + Sync),*
            {
                pub fn new($($driver: ::std::sync::Arc<$driver>),+) -> Self {
                    Self {
                       $(
                           $field: $driver,
                       )+
                        $($id: ::std::marker::PhantomData),*
                    }
                }
            }
        };
}
