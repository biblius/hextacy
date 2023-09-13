use proc_macro_error::abort;
use quote::{format_ident, quote};
use syn::{spanned::Spanned, DeriveInput, LitStr};

pub fn impl_constructor(input: DeriveInput) -> Result<proc_macro2::TokenStream, syn::Error> {
    let syn::Data::Struct(strct) = input.data else {
        abort!(
            input.span(),
            "Constructor derive only works for named structs"
        );
    };

    let struct_id = &input.ident;
    let (im, ty, whe) = input.generics.split_for_impl();
    let mut env_vars = vec![];

    let mut env_field_ids = vec![];
    let mut env_field_types = vec![];

    let mut field_ids = vec![];
    let mut field_types = vec![];

    for field in strct.fields.iter() {
        let field_id = field.ident.as_ref().unwrap_or_else(|| {
            abort!(
                field.span(),
                "Constructor derive only works for named structs"
            )
        });
        field_ids.push(field_id);

        let mut env_found = false;
        for attr in field.attrs.iter() {
            if attr.meta.path().is_ident("env") {
                if env_found {
                    abort!(
                        attr.meta.path().span(),
                        "Only one `env` annotation is supported for Constructor"
                    )
                }
                env_found = true;
                let list = attr.meta.require_list()?;
                let var = list.parse_args::<LitStr>()?;
                env_vars.push(var);
                env_field_ids.push(field_id);
            }
        }

        match field.ty {
            syn::Type::Path(_)
            | syn::Type::Array(_)
            | syn::Type::Reference(_)
            | syn::Type::Slice(_)
            | syn::Type::Tuple(_) => {
                field_types.push(field.ty.clone());
                if env_found {
                    env_field_types.push(field.ty.clone())
                }
            }
            _ => abort!(
                field.ty.span(),
                "Cannot derive Constructor on provided type"
            ),
        }
    }

    let new = quote!(
        impl #im #struct_id #ty #whe {
            pub fn new( #( #field_ids : #field_types ),* ) -> Self {
                Self {
                    #(
                        #field_ids
                    ),*
                }
            }
        }
    );

    let load_from_env = (strct.fields.len() == env_vars.len()).then(|| {
        let conversions = quote_conversions(&field_types);
        quote!(
            impl #im #struct_id #ty #whe {
                pub fn new_from_env() -> Option<Self> {
                    let params = hextacy::env::get_multiple(&[ #( #env_vars ),* ]);

                    #(
                        let #field_ids = params.get( #env_vars ) #conversions
                    )*

                    Some(Self {
                        #(#field_ids),*
                    })
                }
            }
        )
    });

    let fn_ids = env_field_ids
        .iter()
        .map(|id| format_ident!("load_{id}_env"))
        .collect::<Vec<_>>();

    let (conversions, types) = quote_fn_conversions(&env_field_types);
    let loaders = quote!(
        impl #im #struct_id #ty #whe {
            #(
                pub fn #fn_ids() -> Option<#types> {
                    hextacy::env::get(#env_vars).ok() #conversions
                }
            )*
        }
    );

    Ok(quote!(
        #new
        #load_from_env
        #loaders
    ))
}

/// Does not abort, pushes empty conversions to maintain ordering and maps options to their inner value.
fn quote_fn_conversions(
    field_types: &[syn::Type],
) -> (Vec<proc_macro2::TokenStream>, Vec<syn::Type>) {
    let mut conversions = vec![];
    let mut tys = vec![];

    for ty in field_types {
        let syn::Type::Path(p) = ty else {
            conversions.push(quote!());
            tys.push(ty.clone());
            continue;
        };
        let id = &p.path.segments[0];

        if id.ident == "Option" {
            let syn::PathArguments::AngleBracketed(ref args) = id.arguments else {
                conversions.push(quote!());
                tys.push(ty.clone());
                continue;
            };

            let actual = args.args.first().unwrap();
            let syn::GenericArgument::Type(typ) = actual else {
                conversions.push(quote!());
                tys.push(ty.clone());
                continue;
            };

            let syn::Type::Path(p) = typ else {
                conversions.push(quote!());
                tys.push(ty.clone());
                continue;
            };

            let actual = &p.path.segments[0];
            if actual.ident == "String" {
                conversions.push(quote!())
            } else {
                let parse_to = &actual.ident;
                conversions.push(quote!(
                    .and_then(|el| el.parse::<#parse_to>().ok() )
                ))
            }
            tys.push(typ.clone());
            continue;
        }

        if id.ident == "String" {
            conversions.push(quote!())
        } else {
            conversions.push(quote!(
                ?.parse().ok()
            ))
        }
        tys.push(ty.clone());
    }
    (conversions, tys)
}

fn quote_conversions(field_types: &[syn::Type]) -> Vec<proc_macro2::TokenStream> {
    let mut conversions = vec![];
    for ty in field_types {
        let syn::Type::Path(p) = ty else {
            abort!(
                ty.span(),
                "Only owned strings and primitives support the env constructor"
            )
        };
        let id = &p.path.segments[0];

        if id.ident == "Option" {
            let syn::PathArguments::AngleBracketed(ref args) = id.arguments else {
                abort!(id.arguments.span(), "Invalid option type for constructor")
            };

            let actual = args.args.first().unwrap();
            let syn::GenericArgument::Type(ty) = actual else {
                abort!(actual.span(), "Cannot derive constructor for argument")
            };

            let syn::Type::Path(p) = ty else {
                abort!(p.span(), "Cannot derive constructor for argument")
            };

            let actual = &p.path.segments[0];
            if actual.ident == "String" {
                conversions.push(quote!(
                    .cloned();
                ))
            } else {
                let parse_to = &actual.ident;
                conversions.push(quote!(
                    .and_then(|el| el.parse::<#parse_to>().ok() );
                ))
            }
            continue;
        }

        if id.ident == "String" {
            conversions.push(quote!(
                ?.clone();
            ))
        } else {
            conversions.push(quote!(
                ?.parse().ok()?;
            ))
        }
    }
    conversions
}
