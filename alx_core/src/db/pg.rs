#[macro_export]
/// Generates a `Repository` struct with `pub(super)` visibility and derives [RepoAccess]. Useful for reducing overall boilerplate
/// in repository adapters. Accepts the following syntax:
///
/// ```ignore
/// pg_repo! {
///     Conn => "Conn", // Optional, defaults to C => "C"
///     SomeBound => SomeRepository<Conn>,
///     OtherBound => OtherRepository<Conn>,
///     /* ... */
/// }
/// ```
/// The first parameter must match an `ident => literal` pattern in order to have a custom named connection.
/// If the parameter is omitted, the connection generic will be named `C`. This macro also provides a `new()` method
/// whose input is anything that implements `DBConnect` for convenience.
macro_rules! pg_repo {
    ($($id:ident => $bound:path),*) => {
        #[allow(non_snake_case)]
        #[derive(Debug, alx_derive::PgRepo)]
        #[connection = "C"]
        pub(super) struct Repository<A, C, $($id),*>
        where
            A: alx_core::clients::db::DBConnect<Connection = C>,
            $($id: $bound),*
        {
            pub client: alx_core::clients::db::Client<A, C>,
            $($id: ::std::marker::PhantomData<$id>),*
        }

        impl<A, C, $($id),*> Repository<A, C, $($id),*>
        where
            A: alx_core::clients::db::DBConnect<Connection = C>,
            $($id: $bound),*
        {
            pub fn new(client: ::std::sync::Arc<A>) -> Self {
                Self {
                    client: alx_core::clients::db::Client::new(client),
                    $($id: ::std::marker::PhantomData),*
                }
            }
        }
    };

    ($conn:ident => $conn_l:literal, $($id:ident => $bound:path),*) => {
        #[allow(non_snake_case)]
        #[derive(Debug, alx_derive::PgRepo)]
        #[connection = $conn_l]
        pub(super) struct Repository<A, $conn, $($id),*>
        where
            A: alx_core::clients::db::DBConnect<Connection = $conn>,
            $($id: $bound),*
        {
            pub client: alx_core::clients::db::Client<A, $conn>,
            $($id: ::std::marker::PhantomData<$id>),*
        }

        impl<A, $conn, $($id),*> Repository<A, $conn, $($id),*>
        where
            A: alx_core::clients::db::DBConnect<Connection = $conn>,
            $($id: $bound),*
        {
            pub fn new(client: ::std::sync::Arc<A>) -> Self {
                Self {
                    client: alx_core::clients::db::Client::new(client),
                    $($id: ::std::marker::PhantomData),*
                }
            }
        }
    };
}

#[macro_export]
macro_rules! pg_atomic {
    ($($id:ident => $bound:path),*) => {
        #[allow(non_snake_case)]
        #[derive(Debug, alx_derive::PgAtomic)]
        #[connection = "C"]
        pub(super) struct Repository<A, C, $($id),*>
        where
            A: alx_core::clients::db::DBConnect<Connection = C>,
            $($id: $bound),*
        {
            client: alx_core::clients::db::Client<A, C>,
            transaction: alx_core::db::Transaction<C>,
            $($id: ::std::marker::PhantomData<$id>),*
        }

        impl<A, C, $($id),*> Repository<A, C, $($id),*>
        where
            A: alx_core::clients::db::DBConnect<Connection = C>,
            $($id: $bound),*
        {
            pub fn new(client: ::std::sync::Arc<A>) -> Self {
                Self {
                    client: alx_core::clients::db::Client::new(client),
                    transaction: alx_core::db::Transaction::new(None),
                    $($id: ::std::marker::PhantomData),*
                }
            }
        }
    };
}
