use super::{
    extract_conn_attrs, has_connections, insert_replaced, modify_type_generics, modify_where_clause,
};
use crate::{MG_CONNECTION, PG_CONNECTION};
use proc_macro2::{Ident, Span};
use proc_macro_error::abort;
use quote::quote;
use syn::{spanned::Spanned, Generics, ImplGenerics};

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
        tokens.push(quote_pg_repo_access(&impl_generics, &generics, ident));
    }

    if has_mongo {
        tokens.push(quote_mg_repo_access(&impl_generics, &generics, ident));
    }

    quote!(
         #(#tokens)*
    )
}

fn quote_pg_repo_access(
    impl_generics: &ImplGenerics,
    generics: &Generics,
    ident: &Ident,
) -> proc_macro2::TokenStream {
    let __type = syn::Ident::new(PG_CONNECTION, Span::call_site());
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    quote!(
         #[async_trait::async_trait(?Send)]
         impl #impl_generics ::alx_core::db::RepositoryAccess<#__type> for #ident #ty_generics #where_clause {
            async fn connect<'a>(&'a self) -> Result<#__type, ::alx_core::db::DatabaseError> {
                self.postgres.connect().await.map_err(|e| e.into())
            }
         }
    )
}

fn quote_mg_repo_access(
    impl_generics: &ImplGenerics,
    generics: &Generics,
    ident: &Ident,
) -> proc_macro2::TokenStream {
    let __type = syn::Ident::new(MG_CONNECTION, Span::call_site());
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    quote!(
         #[async_trait::async_trait(?Send)]
         impl #impl_generics ::alx_core::db::RepositoryAccess<#__type> for #ident #ty_generics #where_clause {
            async fn connect<'a>(&'a self) -> Result<#__type, ::alx_core::db::DatabaseError> {
                self.mongo.connect().await.map_err(|e| e.into())
            }
         }
    )
}
