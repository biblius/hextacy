mod db;

const ALLOWED_CLIENTS: [&str; 2] = ["postgres", "mongo"];
const PG_CONNECTION: &str = "PgPoolConnection";
const MG_CONNECTION: &str = "ClientSession";

use proc_macro_error::proc_macro_error;

#[proc_macro_derive(AcidRepository, attributes(mongo, postgres))]
#[proc_macro_error]
/// Provides an implementation of `AcidRepositoryAccess<C>` and `Atomic` depending on the
/// provided attributes.
///
/// Accepted attributes and fields are:
///
/// `#[postgres(Connection)]` -> `postgres`, `tx_pg`,
///
/// `#[mongo(Connection)]` -> `mongo`, `tx_mg`,
///
/// Useful for deriving on repository structs with generic connections that only use postgres.
/// The `connection` attribute must be specified and equal to the generic connection parameter
/// of the repository, e.g. if the generic connection is specified as `C` then the attribute must
/// be `#[connection = "C"]`.
///
///
/// Deriving structs MUST have a `postgres`, `mongo` or both fields and they must be a generic client
/// `Client<A, C>` provided in `alx_core::clients::db`.
///
/// The structs, depending on which client they are using, must also contain the `tx_pg` or `tx_mg`
/// fields which will be used to keep track of transactions for the respective client. Transaction fields
/// must be a `RefCell<Option<C>>` (`alx_core::db` provides a `Transaction` type for convenience).
///
/// #### Example
///
/// ```ignore
///
/// #[derive(Debug, AcidRepository)]
/// #[postgres(Pg)]
/// #[mongo(Mg)]
/// pub(super) struct Repository<A, B, Pg, Mg, User>
///   where
///     A: DBConnect<Connection = Pg>,
///     B: DBConnect<Connection = Mg>,
///     User: UserRepository<Pg>,
///     Session: SessionRepository<Mg>,
///  {
///     postgres: Client<A, Pg>,
///     mongo: Client<B, Mg>,
///     tx_pg: Transaction<Pg>,
///     tx_mg: Transaction<Mg>,
///     user: PhantomData<User>,
///     session: PhantomData<Session>,
///  }
/// ```
pub fn derive_atomic_repo(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut ast = syn::parse(input).unwrap();
    db::acid_repo::derive(&mut ast).into()
}

#[proc_macro_derive(Repository, attributes(mongo, postgres))]
#[proc_macro_error]
/// Provides an implementation of `RepositoryAccess<C>` depending on the provided attributes.
///
/// Accepted attributes and fields are:
///
/// `#[postgres(Connection)]` -> `postgres`,
///
/// `#[mongo(Connection)]` -> `mongo`,
///
/// The attributes for the client connections must be specified and equal to a generic connection parameter
/// of the repository, e.g. if a generic connection is specified as `C` then the attribute must be
/// either `#[postgres(C)]` or `#[mongo(C)]` and will be concretised to the designated client connection.
///
/// Deriving structs MUST have a `postgres`, `mongo` or both fields and they must be a generic client
/// `Client<A, C>` provided in `alx_core::clients::db`.
///
/// ```ignore
/// #[derive(Debug, Repository)]
/// #[postgres(Connection)]
/// pub(super) struct Repository<C, Connection, User>
///   where
///     C: DBConnect<Connection = Connection>,
///     User: UserRepository<Connection>
///  {
///     postgres: Client<C, Connection>,
///     user: PhantomData<User>
///  }
/// ```
pub fn derive_repository(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut ast = syn::parse(input).unwrap();
    db::repository::derive(&mut ast).into()
}
