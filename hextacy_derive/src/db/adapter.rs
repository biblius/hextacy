use super::Fields;
use crate::db::{
    insert_replaced, modify_where_clause, process_generics, scan_fields, DIESEL_CONNECTION,
    MONGO_CONNECTION, SEAORM_CONNECTION,
};
use proc_macro2::Ident;
use quote::quote;
use syn::{Generics, ImplGenerics, TypePath};

pub fn derive(ast: &mut syn::DeriveInput) -> proc_macro2::TokenStream {
    let ident = ast.ident.clone();

    let fields = scan_fields(ast);
    let replacements = &fields.replacements;

    let (replaced, mut generics) = process_generics(ast, replacements);

    let trimmed_impls = generics.clone();
    let (impl_generics, _, _) = trimmed_impls.split_for_impl();

    insert_replaced(&mut generics, replaced);
    modify_where_clause(&mut generics, replacements);

    let mut tokens = vec![];

    tokens.push(quote_repo_access(
        DIESEL_CONNECTION,
        &impl_generics,
        &generics,
        &ident,
        &fields,
    ));

    tokens.push(quote_repo_access(
        MONGO_CONNECTION,
        &impl_generics,
        &generics,
        &ident,
        &fields,
    ));

    tokens.push(quote_repo_access(
        SEAORM_CONNECTION,
        &impl_generics,
        &generics,
        &ident,
        &fields,
    ));

    quote!(
         #(#tokens)*
    )
}

fn quote_repo_access(
    connection: &str,
    impl_generics: &ImplGenerics,
    generics: &Generics,
    ident: &Ident,
    fields: &Fields,
) -> proc_macro2::TokenStream {
    let Some(driver_field) = fields.driver_fields.get(connection) else { return quote!() };

    let _type: TypePath = syn::parse_str(connection).unwrap();

    dbg!(&_type);

    let (_, ty_generics, where_clause) = generics.split_for_impl();

    quote!(
         #[async_trait::async_trait]
         impl #impl_generics ::hextacy::db::RepositoryAccess<#_type> for #ident #ty_generics #where_clause {
            async fn connect<'a>(&'a self) -> Result<#_type, ::hextacy::db::DatabaseError> {
                self.#driver_field.connect().await.map_err(|e| e.into())
            }
         }
    )
}
