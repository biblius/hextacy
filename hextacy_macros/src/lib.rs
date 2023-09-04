use proc_macro2::Span;
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use syn::{spanned::Spanned, DeriveInput, Ident, ItemImpl, TypePath};

mod configuration;

/// Intended to be used on configuration/state structs that need to instantiate themselves using variables obtained
/// from an external source.
///
/// This macro assumes a constructor (an associated function named `new`) exists for the annotated field's type and
/// creates a function that calls it using the specified keys and strategy. If a constructor does not exist, you can
/// specify an associated function of the struct to load with `#[load_with(T::some_constructor)]`.
///
/// If loaders are configured on all of the struct's fields, it will receive an associated function `configure()`
/// which can be used to construct it via the generated functions. If you need fine tuning, you can use the
/// generated functions in a custom function and create your own loading logic.
///
/// For each field annotated and for each annotation, the function's signature will be
/// `fn init_<field_name>_<strategy>() { .. }`. See strategies below in field annotations.
///
/// Each field can also be wrapped, e.g. in an `Arc`, but a few rules apply here:
/// - Wrappers can be nested any amount, but they must solely consist of single argument angle-bracket wrappers
/// - The wrappers must have an associated `new` function (constructor)
/// - The wrapper cannot be `Option`
///
/// If a field constructor is async, the field must be annotated with `#[load_async]` to support it. If any of the
/// constructors are async, the resulting `configure()` function will be async as well.
///
/// ## Field annotations
///
/// The order of field annotations specifies the priority of loading the variables. Each subsequent annotation will be a fallback
/// strategy to the previous if it fails. If all the strategies fail for a given field, the `configure()` function will return an error.
///
/// The order of variables specified, as well as the types, must match the order of the constructor's signature.
///
/// Examples use the following dummy adapter:
///
/// ```ignore
/// struct DummyAdapter {
///     // ...
/// }
///
/// // The order of the variables must match the variables in field annotations
/// impl DummyAdapter {
///     pub fn new(host: &str, port: u16, pool_size: Option<u16>) -> DummyAdapter {
///         // ...
///     }
///     pub async fn new_async(host: &str, port: u16, pool_size: Option<u16>) -> DummyAdapter {
///         // ...
///     }
/// }
/// ```
///
/// ### `env`
///
/// - All variables are loaded as `String`s by default and are passed as `&str`s to the constructors
/// - Variables are required by default, the function will return an error if it cannot find the variable in the env
/// - Variables can be optional by appending `as Option`, e.g. `"MY_VAR" as Option`
/// - Variables can be parsed by appending `as T`, e.g. `"MY_VAR" as usize`
/// - Both can be applied by appending `as Option<T>`, e.g. `"MY_VAR" as Option<u16>`
///  
/// The function generated calls `hextacy::config::env::get_multiple`, parses the variables if specified and calls the
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
///     pub postgres: Arc<Mutex<MyPgAdapter>>
/// }
/// ```
///
/// ### `raw`
///
/// - Call constructor with the specified values
/// - Generated function never errors
///
/// #### Example
///
/// ```ignore
/// use hextacy::Configuration;
///
/// #[derive(Debug, Configuration)]
/// struct MyAppState {
///     #[raw("localhost", 5432, Some(8))]
///     pub postgres: Arc<MyPgAdapter>
/// }
/// ```
///
/// ### `load_async`
///
/// - Use this when the constructor is async
///
/// ### `load_with`
///
/// - Use this to specify an associated function to call instead of `new`
///
/// #### Examples
///
/// ```ignore
/// #[derive(Debug, Configuration)]
/// struct MyAppState {
///     #[env(
///         "HOST",
///         "PORT" as u16,
///         "POOL_SIZE" as Option<u16>
///     )]
///     // Raw is used as a fallback here
///     #[raw("localhost", 5432, Some(8))]
///     #[load_async]
///     #[load_with(MyPgAdapter::new_async)]
///     pub postgres: Arc<MyPgAdapter>
/// }
/// ````
#[proc_macro_derive(Configuration, attributes(env, raw, load_async, load_with))]
#[proc_macro_error]
pub fn derive_config(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();
    configuration::impl_configuration(input)
        .expect("Error while parsing Configuration")
        .into()
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
