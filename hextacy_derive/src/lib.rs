mod db;

use proc_macro2::Span;
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use syn::{spanned::Spanned, Ident, ItemImpl, TypePath};

/// Provides an implementation of `RepositoryAccess<C>` depending on the provided attributes.
///
/// Intended to be derived on service components that contain a generic connection that needs to
/// be concretised. The macro will essentially replace the generic connection with a concrete one
/// used by the driver in the aformentioned implementation.
///
/// Multiple driver fields on the same struct are allowed.
///
/// Accepted field attributes (drivers) are:
///
/// `diesel`,
/// `mongo`,
/// `seaorm`
///
/// Useful for deriving on repository components with generic connections.
/// The `driver` field attribute must be specified on a `Driver` field and match
/// the generic connection parameter of the component, e.g. if the generic connection
/// is specified as `C` then the field attribute must be `#[driver(C)]`.
///
/// If using the `component!` macro, this will be done automatically behind the scenes
/// for whatever drivers are passed in as the parameters to the macro.
///
/// ```ignore
/// #[derive(Debug, Adapter)]
/// pub(super) struct RepositoryComponent<C, Connection, User>
///   where
///     C: DBConnect<Connection = Connection>,
///     User: UserRepository<Connection>
///  {
///     #[diesel(Connection)]
///     driver: Driver<C, Connection>,
///     user: PhantomData<User>
///  }
/// ```
#[proc_macro_derive(Adapter, attributes(diesel, seaorm, mongo))]
#[proc_macro_error]
pub fn derive_adapter(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut ast = syn::parse(input).unwrap();
    db::adapter::derive(&mut ast).into()
}

#[proc_macro_attribute]
#[proc_macro_error]
/// When deriving on an impl block for a struct, this will instead create a trait whose name
/// is the original struct name suffixed with `Api` and implements it on the struct. The trait
/// has the same signatures as the functions in the impl block.
///
/// This allows for easy mocking of component APIs for unit testing, as well as for DI through bounds
/// on services.
///
/// Visibility can be provided for the generated trait, e.g. `#[component(crate)]`
pub fn component(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let ast: ItemImpl = syn::parse(input.clone()).unwrap();

    let (impl_generics, type_generics, where_clause) = ast.generics.split_for_impl();

    let (original_struct, trait_ident) = match ast.self_ty.as_ref() {
        syn::Type::Path(TypePath { ref path, .. }) => {
            let struct_name = &path.segments[0].ident;
            (
                struct_name,
                Ident::new(&format!("{}Api", struct_name), Span::call_site()),
            )
        }
        _ => abort!(ast.span(), "component not supported for this type of impl"),
    };

    let mut fn_defs = vec![];

    let original_fns = ast
        .items
        .iter()
        .map(|item| {
            let syn::ImplItem::Fn(func) = item else {
                abort!(item.span(), "component not supported for this type of impl")
            };
            let sig = &func.sig;
            let tokens = quote!(#sig ;);
            fn_defs.push(tokens);
            func
        })
        .collect::<Vec<_>>();

    let visibility: Option<proc_macro2::TokenStream> = (!attr.is_empty()).then(|| {
        let attr: proc_macro2::TokenStream = attr.into();
        quote! {(in #attr)}
    });

    quote!(
        #[cfg_attr(test, mockall::automock)]
        #[async_trait::async_trait]
        pub #visibility trait #trait_ident {
            #(#fn_defs)*
        }

        #[async_trait::async_trait]
        impl #impl_generics #trait_ident for #original_struct #type_generics #where_clause {
            #(#original_fns)*
        }
    )
    .into()
}

/*             if let Ok(mut file) = std::fs::read_to_string("./fn") {
    file.push_str("\n\n\n");
    file.push_str(tokens.to_string().as_str());
    std::fs::write("./fn", file).unwrap();
} else {
    std::fs::write("./fn", tokens.to_string()).unwrap();
} */
