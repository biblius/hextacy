#[macro_export]
/// Generates a `Repository` struct (or a custom name) with `pub(super)` visibility and derives [RepositoryAccess]
/// [super::RepositoryAccess] with either a postgres or mongo connection or both.
///
/// Useful for reducing overall boilerplate in repository adapters.
///
/// The fields must be named `postgres` and `mongo` if they are used as they are forwarded to
/// the derive macro for processing. The fields are also used for the attribute meta in the derive
/// macro. The generated repository struct will have `RepositoryAccess` derived for any of the specified
/// connections.
///
/// The clients module includes `Postgres` and `Mongo` which derive DBConnect for the derived
/// connections. Check out [Repository][alx_derive::Repository]
///
/// Accepts the following syntax:
///
/// ```ignore
/// repository! {
///     Postgres => PgConnection : postgres,
///     Mongo => MgConnection : mongo;
///
///     SomeBound => SomeRepository<Conn>,
///     OtherBound => OtherRepository<Conn>,
///     /* ... */
/// }
/// ```
///
/// This macro also provides a `new()` method whose input is anything that implements `DBConnect` for convenience.
/// `DBConnect` is automatically added to the bounds as a generic parameter for the client.
macro_rules! repository {
    (
      $($client:ident => $conn_name:ident : $field:ident),+;
      $($id:ident => $repo_bound:path),*
    ) => {
         #[allow(non_snake_case)]
         #[derive(Debug, Clone, alx_derive::Repository)]
         $(
            #[$field($conn_name)]
         )+
         pub struct Repository<$($client),+, $($conn_name),+, $($id),*>
         where
            $(
                $client: alx_core::clients::db::DBConnect<Connection = $conn_name>,
            )+
             $($id: $repo_bound),*
         {
            $(
                pub $field: alx_core::clients::db::Client<$client, $conn_name>,
            )+
             $($id: ::std::marker::PhantomData<$id>),*
         }

         #[allow(non_snake_case)]
         impl<$($client),+, $($conn_name),+, $($id),*> Repository<$($client),+, $($conn_name),+, $($id),*>
         where
            $(
                $client: alx_core::clients::db::DBConnect<Connection = $conn_name>,
            )+
             $($id: $repo_bound),*
         {
             pub fn new($($client: ::std::sync::Arc<$client>),+) -> Self {
                 Self {
                    $(
                        $field: alx_core::clients::db::Client::new($client),
                    )+
                     $($id: ::std::marker::PhantomData),*
                 }
             }
         }
    };
}

#[macro_export]
/// Generates a `Repository` struct (or a custom name) with `pub(super)` visibility and derives [AcidRepositoryAccess]
/// [super::AcidRepositoryAccess]. Useful for reducing overall boilerplate in repository adapters.
///
/// Accepts the following syntax:
///
/// ```ignore
/// acid_repo! {
///     RepoName, // Optional, defaults to Repository
///
///     Conn => "Conn", // Optional, defaults to C => "C"
///
///     SomeBound => SomeRepository<Conn>,
///     OtherBound => OtherRepository<Conn>,
///     /* ... */
/// }
/// ```
/// The first parameter is optional and if provided will rename the generated struct to the value.
///
/// The second parameter must match an `ident => literal` pattern in order to have a custom named connection.
/// This is due to the generic parameters in the repositories and the derive macro for Repository.
/// If the parameter is omitted, the connection generic will be named `C`.
///
/// This macro also provides a `new()` method whose input is anything that implements `DBConnect` for convenience.
/// `DBConnect` is automatically added to the bounds as a generic parameter for the client.
macro_rules! acid_repo {
    (
      $($client:ident => $conn_name:ident : $field:ident, $tx:ident),+;
      $($id:ident => $repo_bound:path),*
    ) => {
           #[allow(non_snake_case)]
           #[derive(Debug, Clone, alx_derive::AcidRepository)]
           $(
              #[$field($conn_name)]
           )+
           pub struct Repository<$($client),+, $($conn_name),+, $($id),*>
           where
              $(
                  $client: alx_core::clients::db::DBConnect<Connection = $conn_name>,
              )+
               $($id: $repo_bound),*
           {
              $(
                  $field: alx_core::clients::db::Client<$client, $conn_name>,
                  $tx: alx_core::db::Transaction<$conn_name>,
              )+
               $($id: ::std::marker::PhantomData<$id>),*
           }

           #[allow(non_snake_case)]
           impl<$($client),+, $($conn_name),+, $($id),*> Repository<$($client),+, $($conn_name),+, $($id),*>
           where
              $(
                  $client: alx_core::clients::db::DBConnect<Connection = $conn_name>,
              )+
               $($id: $repo_bound),*
           {
               pub fn new($($client: ::std::sync::Arc<$client>),+) -> Self {
                   Self {
                      $(
                          $field: alx_core::clients::db::Client::new($client),
                          $tx: alx_core::db::Transaction::new(None),
                      )+
                       $($id: ::std::marker::PhantomData),*
                   }
               }
           }
      };
}
