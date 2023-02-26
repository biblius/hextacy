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
         impl #impl_generics ::alx_core::db::RepoAccess<#__type> for #ident #ty_generics #where_clause {
            fn connect<'a>(&'a self) -> Result<#__type, ::alx_core::db::DatabaseError> {
                self.client.connect().map_err(|e| e.into())
            }
         }
    )
}
