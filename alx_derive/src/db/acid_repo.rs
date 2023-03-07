use crate::{db::has_connections, MG_CONNECTION, PG_CONNECTION};

use super::{extract_conn_attrs, insert_replaced, modify_type_generics, modify_where_clause};
use proc_macro2::Span;
use proc_macro_error::abort;
use quote::quote;
use syn::{spanned::Spanned, Generics, Ident, ImplGenerics};

pub fn derive(ast: &mut syn::DeriveInput) -> proc_macro2::TokenStream {
    let ident = &ast.ident;

    // Extract the connection type
    let connection_attrs = extract_conn_attrs(&ast).unwrap();
    let (has_pg, has_mongo) = has_connections(&connection_attrs);

    // Clone the generics as we do not want to modify the input
    let mut generics = ast.generics.clone();

    // Modify the type generics by taking out the specified connection generic from the impl and substitute
    // it in the following bounds with whatever they are paired with
    let replaced = modify_type_generics(&mut generics, &connection_attrs);
    if replaced.is_empty() {
        abort!(
            generics.span(),
            format!("Cannot find the specified connection type {connection_attrs:?}");
            help = "Make sure the connection name matches the generic on the struct"
        );
    }

    let trimmed_impls = generics.clone();
    let (impl_generics, _, _) = trimmed_impls.split_for_impl();

    insert_replaced(&mut generics, replaced);
    modify_where_clause(&mut generics, &connection_attrs);

    let mut tokens = vec![];
    if has_pg {
        tokens.push(quote_pg_atomic_access(&impl_generics, &generics, ident));
    }

    if has_mongo {
        tokens.push(quote_mg_atomic_access(&impl_generics, &generics, ident));
    }

    tokens.push(quote_atomic_access(
        &impl_generics,
        &generics,
        ident,
        (has_pg, has_mongo),
    ));
    quote!(
         #(#tokens)*
    )
}

fn quote_pg_atomic_access(
    impl_generics: &ImplGenerics,
    generics: &Generics,
    ident: &Ident,
) -> proc_macro2::TokenStream {
    let __type = syn::Ident::new(PG_CONNECTION, Span::call_site());
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    quote!(
        #[async_trait::async_trait(?Send)]
         impl #impl_generics ::alx_core::db::AcidRepositoryAccess<#__type> for #ident #ty_generics #where_clause {
            async fn connect<'a>(&'a self) -> Result<alx_core::db::AtomicConnection<#__type>, alx_core::db::DatabaseError> {
                let tx = self.tx_pg.borrow_mut();
                match *tx {
                    Some(ref _tx) => Ok(::alx_core::db::AtomicConnection::Existing(tx)),
                    None => {
                        let conn = self.postgres.connect().await?;
                        Ok(alx_core::db::AtomicConnection::New(conn))
                    },
                }
            }
         }
    )
}

fn quote_mg_atomic_access(
    impl_generics: &ImplGenerics,
    generics: &Generics,
    ident: &Ident,
) -> proc_macro2::TokenStream {
    let __type = syn::Ident::new(MG_CONNECTION, Span::call_site());
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    quote!(
        #[async_trait::async_trait(?Send)]
         impl #impl_generics ::alx_core::db::AcidRepositoryAccess<#__type> for #ident #ty_generics #where_clause {
            async fn connect<'a>(&'a self) -> Result<alx_core::db::AtomicConnection<#__type>, alx_core::db::DatabaseError> {
                let tx = self.tx_mg.borrow_mut();
                match *tx {
                    Some(ref _tx) => Ok(::alx_core::db::AtomicConnection::Existing(tx)),
                    None => {
                        let conn = self.mongo.connect().await?;
                        Ok(alx_core::db::AtomicConnection::New(conn))
                    },
                }
            }
         }
    )
}

fn quote_atomic_access(
    impl_generics: &ImplGenerics,
    generics: &Generics,
    ident: &Ident,
    (has_pg, has_mg): (bool, bool),
) -> proc_macro2::TokenStream {
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let start = quote_atomic_start(has_pg, has_mg);
    let commit = quote_atomic_commit(has_pg, has_mg);
    let abort = quote_atomic_abort(has_pg, has_mg);
    quote!(
        #[async_trait::async_trait(?Send)]
        impl #impl_generics ::alx_core::db::Atomic for #ident #ty_generics #where_clause {
            async fn start_transaction(&self) -> Result<(), alx_core::db::DatabaseError> {
                #start
                Ok(())
            }
            async fn commit_transaction(&self) -> Result<(), alx_core::db::DatabaseError> {
                #commit
                Ok(())
            }
            async fn rollback_transaction(&self) -> Result<(), alx_core::db::DatabaseError> {
                #abort
                Ok(())
            }
        }
    )
}

fn quote_atomic_start(has_pg: bool, has_mg: bool) -> proc_macro2::TokenStream {
    let pg = quote!(
            let mut tx = self.tx_pg.borrow_mut();
            match *tx {
                Some(_) => return Err(alx_core::db::DatabaseError::Transaction(::alx_core::db::TransactionError::InProgress)),
                None => {
                    let mut conn = self.postgres.connect().await?;
                    diesel::connection::AnsiTransactionManager::begin_transaction(&mut *conn)?;
                    *tx = Some(conn);
                }
            };
    );

    let mg = quote!(
        let mut tx = self.tx_mg.borrow_mut();
        match *tx {
            Some(_) => {
                return Err(alx_core::db::DatabaseError::Transaction(
                    ::alx_core::db::TransactionError::InProgress,
                ))
            },
            None => {
                let mut conn = self.mongo.connect().await?;
                conn.start_transaction(None).await?;
                *tx = Some(conn);
            }
        };
    );

    if has_pg && has_mg {
        quote!(
            #pg
            #mg
        )
    } else if has_pg {
        pg
    } else {
        mg
    }
}

fn quote_atomic_commit(has_pg: bool, has_mg: bool) -> proc_macro2::TokenStream {
    let pg = quote!(
        let mut tx = self.tx_pg.borrow_mut();
        match tx.take() {
            Some(ref mut conn) => {
                diesel::connection::AnsiTransactionManager::commit_transaction(&mut **conn)
                    .map_err(alx_core::db::DatabaseError::from)?;
            },
            None => return Err(alx_core::db::DatabaseError::Transaction(
                ::alx_core::db::TransactionError::NonExisting,
            )
            .into()),
        };
    );

    let mg = quote!(
        let mut tx = self.tx_mg.borrow_mut();
        match tx.take() {
            Some(ref mut conn) => {
                conn.commit_transaction().await?;
            },
            None => {
                return Err(alx_core::db::DatabaseError::Transaction(
                    ::alx_core::db::TransactionError::NonExisting,
                )
                .into())
            }
        };
    );

    if has_pg && has_mg {
        quote!(
            #pg
            #mg
        )
    } else if has_pg {
        pg
    } else {
        mg
    }
}

fn quote_atomic_abort(has_pg: bool, has_mg: bool) -> proc_macro2::TokenStream {
    let pg = quote!(
        let mut tx = self.tx_pg.borrow_mut();
        match tx.take() {
            Some(ref mut conn) => {
                ::diesel::connection::AnsiTransactionManager::rollback_transaction(&mut **conn)
                    .map_err(alx_core::db::DatabaseError::from)?;
            },
            None => return Err(alx_core::db::DatabaseError::Transaction(
                ::alx_core::db::TransactionError::NonExisting,
            )
            .into()),
        };
    );

    let mg = quote!(
        let mut tx = self.tx_mg.borrow_mut();
        match tx.take() {
            Some(ref mut conn) => conn.abort_transaction().await?,
            None => {
                return Err(alx_core::db::DatabaseError::Transaction(
                    ::alx_core::db::TransactionError::NonExisting,
                )
                .into())
            }
        };
    );

    if has_pg && has_mg {
        quote!(
            #pg
            #mg
        )
    } else if has_pg {
        pg
    } else {
        mg
    }
}
