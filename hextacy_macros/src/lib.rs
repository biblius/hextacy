use proc_macro2::Span;
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use syn::{spanned::Spanned, Ident, ItemImpl, TypePath};

#[proc_macro_attribute]
#[proc_macro_error]
/// When annotating an impl block for a struct, this will instead create a trait whose name
/// is the original struct name suffixed with `Contract` and implements it on the struct. The trait
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
