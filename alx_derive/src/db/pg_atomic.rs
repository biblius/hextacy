use super::{
    extract_conn_attr, insert_replaced, modify_type_generics, modify_where_clause, CONN_ATTR,
};
use proc_macro2::Span;
use proc_macro_error::abort;
use quote::quote;
use syn::spanned::Spanned;

pub fn derive(ast: &mut syn::DeriveInput, conn: &str) -> proc_macro2::TokenStream {
    let ident = &ast.ident;

    // Extract the connection type
    let connection_attr = extract_conn_attr(&ast, CONN_ATTR).unwrap();

    // Clone the generics as we do not want to modify the input
    let mut generics = ast.generics.clone();

    // Modify the type generics by taking out the specified connection generic from the impl and substitute
    // it in the following bounds with PgPoolConnection
    let replaced = modify_type_generics(&mut generics, &connection_attr, conn);
    if replaced.is_empty() {
        abort!(
            generics.span(),
            format!("Cannot find the specified connection type {connection_attr:?}");
            help = "Make sure the connection name matches the generic on the struct"
        );
    }

    let trimmed_impls = generics.clone();
    let (impl_generics, _, _) = trimmed_impls.split_for_impl();

    insert_replaced(&mut generics, replaced);
    modify_where_clause(&mut generics, &connection_attr, conn);

    let (_, ty_generics, where_clause) = generics.split_for_impl();

    let __type = syn::Ident::new(conn, Span::call_site());

    quote!(
         impl #impl_generics ::alx_core::db::AtomicRepoAccess<#__type> for #ident #ty_generics #where_clause {
            fn connect<'a>(&'a self) -> Result<::alx_core::db::AtomicConn<#__type>, alx_core::db::DatabaseError> {
                let tx = self.transaction.borrow_mut();
                match *tx {
                    Some(ref _tx) => Ok(::alx_core::db::AtomicConn::Existing(tx)),
                    None => Ok(::alx_core::db::AtomicConn::New(self.client.connect()?)),
                }
            }
         }

         impl #impl_generics ::alx_core::db::Atomic for #ident #ty_generics #where_clause {
            fn start_transaction(&self) -> Result<(), alx_core::db::DatabaseError> {
                let mut tx = self.transaction.borrow_mut();
                match *tx {
                    Some(_) => Err(alx_core::db::DatabaseError::Transaction(::alx_core::db::TransactionError::InProgress)),
                    None => {
                        let mut conn = self.client.connect()?;
                        diesel::connection::AnsiTransactionManager::begin_transaction(&mut *conn)?;
                        *tx = Some(conn);
                        Ok(())
                    }
                }
            }

            fn rollback_transaction(&self) -> Result<(), alx_core::db::DatabaseError> {
                let mut tx = self.transaction.borrow_mut();
                match tx.take() {
                    Some(ref mut conn) => ::diesel::connection::AnsiTransactionManager::rollback_transaction(&mut **conn)
                        .map_err(alx_core::db::DatabaseError::from),
                    None => Err(alx_core::db::DatabaseError::Transaction(::alx_core::db::TransactionError::NonExisting).into()),
                }
            }

            fn commit_transaction(&self) -> Result<(), alx_core::db::DatabaseError> {
                let mut tx = self.transaction.borrow_mut();
                match tx.take() {
                    Some(ref mut conn) => {
                        diesel::connection::AnsiTransactionManager::commit_transaction(&mut **conn)
                            .map_err(alx_core::db::DatabaseError::from)
                    }
                    None => Err(alx_core::db::DatabaseError::Transaction(::alx_core::db::TransactionError::NonExisting).into()),
                }
            }
        }
    )
}
