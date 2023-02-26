use proc_macro_error::proc_macro_error;

mod db;

const PG_POOLED: &str = "PgPoolConnection";

#[proc_macro_derive(PgAtomic, attributes(connection))]
#[proc_macro_error]
/// Provides an implementation of `AtomicRepoAccess<PgPoolConnection>` and `Atomic`.
///
/// Useful for deriving on repository structs with generic connections. The `connection` attribute
/// must be specified and equal to the generic connection parameter of the repository, e.g. if the generic
/// connection is specified as `C` then the attribute must be `#[connection = "C"]`.
///
/// Deriving structs MUST have a `client` and `transaction` field where the `client` is a generic database client
/// `Client<A, C>` provided in `alx_core::clients::db` and the transaction is a `RefCell<Option<C>>`
/// (`alx_core::db` provides a `Transaction` type for convenience).
///
/// ```ignore
/// use alx_core::{clients::db::Client, db::Transaction};
///
/// #[derive(Debug, PgAtomic)]
/// #[connection = "C"]
/// pub(super) struct Repository<A, C, User>
///   where
///     A: DBConnect<Connection = C>,
///     User: UserRepository<C>
///  {
///     pub client: Client<A, C>,
///     pub transaction: Transaction<C>,
///     _user: User,
///  }
/// ```
pub fn derive_pg_atomic(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut ast = syn::parse(input).unwrap();
    db::pg_atomic::derive(&mut ast, PG_POOLED).into()
}

#[proc_macro_derive(PgRepo, attributes(connection))]
#[proc_macro_error]
/// Provides an implementation of `RepoAccess<PgPoolConnection>`.
///
/// Useful for deriving on repository structs with generic connections. The `connection` attribute
/// must be specified and equal to the generic connection parameter of the repository, e.g. if the generic
/// connection is specified as `C` then the attribute must be `#[connection = "C"]`.
///
/// Deriving structs MUST have a `client` field where the `client` is a generic database client
/// `Client<A, C>` provided in `alx_core::clients::db`.
///
/// ```ignore
/// #[derive(Debug, PgRepo)]
/// #[connection = "C"]
/// pub(super) struct Repository<A, C, User>
///   where
///     A: DBConnect<Connection = C>,
///     User: UserRepository<C>
///  {
///     pub client: Client<A, C>,
///     _user: User
///  }
/// ```
pub fn derive_pg_pooled(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut ast = syn::parse(input).unwrap();
    db::pg::derive(&mut ast, PG_POOLED).into()
}
