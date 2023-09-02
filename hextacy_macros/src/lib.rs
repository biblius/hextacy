use proc_macro2::Span;
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use syn::{spanned::Spanned, DeriveInput, Ident, ItemImpl, TypePath};

mod configuration;

/// Intended to be used on configuration/state structs that need to instantiate themselves using env variables.
///
/// This macro assumes a constructor (an associated function named `new`) exists for the annotated field's type and
/// creates a function that initialises said struct using the specified keys and strategy.
///
/// For each field annotated and for each annotation, the function's signature will be
/// `fn init_<field_name>_<strategy>() { .. }`.
///
/// Each field can also be wrapped, e.g. in an `Arc`, but a few rules apply here:
/// - Wrappers can be nested any amount, but they must solely consist of single argument angle-bracket wrappers
/// - The wrappers must have an associated `new` function (constructor)
/// - The wrapper cannot be `Option`
///
/// ## Field annotations
///
/// ### `env`
///
/// - The order of variables specified, as well as the types, must match the order of the constructor's signature.
/// - All variables are loaded as `String`s by default
/// - Variables can be parsed by appending `as T`, e.g. `"MY_VAR" as usize`
/// - Variables can be optional by appending `as Option`, e.g. `"MY_VAR" as Option`
/// - Variables can be both parsed and optional by appending `as Option<T>`, e.g. `"MY_VAR" as Option<u16>`
///  
/// The function generated calls `hextacy::env::get_multiple`, parses the variables if specified and calls the
/// struct's constructor.
///
/// #### Example
///
/// ```ignore
/// use hextacy::Configuration;
///
/// #[derive(Debug, Configuration)]
/// struct MyAppState {
///     // Must follow the order of the variables in the constructor of MyPgAdapter
///     #[env(
///         "HOST",
///         "PORT" as u16,
///         "POOL_SIZE" as Option<u16>
///     )]
///     // Mutex is just for example, don't do this at home
///     pub postgres: Arc<Mutex<MyPgAdapter>>
/// }
///
/// struct DummyAdapter {
///     // ...
/// }
///
/// impl DummyAdapter {
///     // The order of the variables here determines how the
///     pub fn new(host: &str, port: u16, pool_size: Option<u16>) -> DummyAdapter {
///         // ...
///     }
/// }
///
/// ### `raw`
///
/// - The order of variables specified, as well as the types, must match the order of the constructor's signature.
/// ```
#[proc_macro_derive(Configuration, attributes(env, raw))]
#[proc_macro_error]
pub fn derive_config(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = syn::parse(input.clone()).unwrap();
    configuration::impl_configuration(input).into()
}

#[proc_macro_attribute]
#[proc_macro_error]
/// When annotating an impl block for a struct, this will instead create a trait whose name
/// is the original struct name suffixed with `Contract` and implement it on the struct. The trait
/// has the same signatures as the functions in the impl block.
///
/// This allows for easy mocking of component contracts for unit testing, as well as for DI through bounds
/// on services.
///
/// Visibility can be provided for the generated trait, e.g. `#[contract(crate)]`
pub fn contract(
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
                Ident::new(&format!("{}Contract", struct_name), Span::call_site()),
            )
        }
        _ => abort!(
            ast.self_ty.span(),
            "contract not supported for this type of impl"
        ),
    };

    let mut fn_defs = vec![];

    let original_fns = ast
        .items
        .iter()
        .map(|item| {
            let syn::ImplItem::Fn(func) = item else {
                abort!(item.span(), "contract not supported for this type of impl")
            };

            let sig = &func.sig;
            let tokens = quote!(#sig ;);
            fn_defs.push(tokens);
            func
        })
        .collect::<Vec<_>>();

    let visibility: Option<proc_macro2::TokenStream> = (!attr.is_empty()).then(|| {
        let attr: proc_macro2::TokenStream = attr.into();
        quote! { (in #attr) }
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
