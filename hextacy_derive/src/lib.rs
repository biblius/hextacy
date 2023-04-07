mod db;

use proc_macro_error::proc_macro_error;

#[proc_macro_derive(Repository, attributes(diesel, seaorm, mongo))]
#[proc_macro_error]
/// Provides an implementation of `RepositoryAccess<C>` depending on the provided attributes.
///
/// Accepted field attributes are:
///
/// `diesel`,
/// `mongo`,
/// `seaorm`
///
/// Useful for deriving on repository structs with generic connections.
/// The `driver` field attribute must be specified on a `Driver` field and match
/// the generic connection parameter of the repository, e.g. if the generic connection
/// is specified as `C` then the field attribute must be `#[driver(C)]`.
///
/// ```ignore
/// #[derive(Debug, Repository)]
/// #[postgres(Connection)]
/// pub(super) struct Repository<C, Connection, User>
///   where
///     C: DBConnect<Connection = Connection>,
///     User: UserRepository<Connection>
///  {
///     postgres: Driver<C, Connection>,
///     user: PhantomData<User>
///  }
/// ```
pub fn derive_repository(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut ast = syn::parse(input).unwrap();
    db::repository::derive(&mut ast).into()
}
