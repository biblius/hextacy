pub mod pg;
pub mod pg_atomic;

use proc_macro2::Span;
use proc_macro_error::abort;
use syn::{
    punctuated::Punctuated,
    spanned::Spanned,
    token::{Add, Comma},
    GenericParam, Generics, TypeParamBound,
};

const CONN_ATTR: &str = "connection";

/// Modifies the generic connection type to the concrete implementation
/// given by `conn_replace` and returns the replaced trait bounds and their indices.
///
/// This function is used to remove the generic parameter from the arguments directly following
/// `impl`.
fn modify_type_generics(
    generics: &mut Generics,
    conn_attr: &str,
    conn_replace: &str,
) -> Vec<(usize, GenericParam)> {
    let mut replaced = vec![];
    let types = generics
        .type_params_mut()
        .enumerate()
        .filter_map(|(i, param)| {
            if param.ident.to_string() != conn_attr {
                modify_bounds(&mut param.bounds, conn_attr, conn_replace);
                Some(GenericParam::Type(param.clone()))
            } else {
                let mut param = param.clone();
                param.ident = syn::Ident::new(conn_replace, param.span());
                replaced.push((i, GenericParam::Type(param)));
                None
            }
        })
        .collect::<Punctuated<GenericParam, Comma>>();

    generics.params = types;
    replaced
}

/// Re-adds the replaced trait bounds after the initial impl block
fn insert_replaced(generics: &mut Generics, replaced: Vec<(usize, GenericParam)>) {
    for (i, rep) in replaced {
        generics.params.insert(i, rep)
    }
}

/// Modify the where clause of the impl by substituting generic connection bounds with the concrete one
fn modify_where_clause(generics: &mut Generics, conn_attr: &str, conn_replace: &str) {
    let where_predicates = &mut generics.make_where_clause().predicates;

    for pred in where_predicates.iter_mut() {
        let syn::WherePredicate::Type(_ty) = pred else { continue; };

        modify_bounds(&mut _ty.bounds, conn_attr, conn_replace);
    }
}

/// Replace `conn_attr` generic bounds with the concrete one
fn modify_bounds(
    bounds: &mut Punctuated<TypeParamBound, Add>,
    conn_attr: &str,
    conn_replace: &str,
) {
    for bound in bounds.iter_mut() {
        let syn::TypeParamBound::Trait(ref mut tr) = bound else { continue; };

        for seg in tr.path.segments.iter_mut() {
            if seg.ident.to_string() == conn_attr {
                seg.ident = syn::Ident::new(conn_replace, seg.span())
            }

            let syn::PathArguments::AngleBracketed(ref mut ab_args) = seg.arguments else { continue; };

            for arg in ab_args.args.iter_mut() {
                match arg {
                    syn::GenericArgument::Type(ref mut ty) => match ty {
                        syn::Type::Path(ref mut p) => {
                            for seg in p.path.segments.iter_mut() {
                                if seg.ident.to_string() == conn_attr {
                                    seg.ident = syn::Ident::new(conn_replace, seg.span())
                                }
                            }
                        }
                        _ => {}
                    },
                    syn::GenericArgument::Binding(ref mut bind) => match bind.ty {
                        syn::Type::Path(ref mut path) => {
                            for seg in path.path.segments.iter_mut() {
                                if seg.ident.to_string() == conn_attr {
                                    seg.ident = syn::Ident::new(conn_replace, seg.span())
                                }
                            }
                        }
                        _ => {}
                    },
                    // This should theoretically be unreachable
                    syn::GenericArgument::Const(_)
                    | syn::GenericArgument::Lifetime(_)
                    | syn::GenericArgument::Constraint(_) => unimplemented!(),
                }
            }
        }
    }
}

fn extract_conn_attr(ast: &syn::DeriveInput, name: &str) -> syn::Result<String> {
    let attr = ast
        .attrs
        .iter()
        .find_map(|a| {
            let meta = a
                .parse_meta()
                .map_err(|_| {
                    abort!(
                        a.span(),
                        "Couldn't parse meta attribute";
                        help = "Make sure the value is a string literal"
                    )
                })
                .unwrap();
            if meta.path().is_ident(name) {
                Some(meta)
            } else {
                None
            }
        })
        .unwrap_or_else(|| {
            abort!(
                Span::call_site(),
                "#[derive(Repo)] must have a #[connection = \"...\"] attribute"
            );
        });

    'f: {
        if let syn::Meta::NameValue(ref val) = attr {
            match val.lit {
                syn::Lit::Str(ref str) => return Ok(str.token().to_string().replace("\"", "")),
                _ => break 'f,
            }
        }
    }
    abort!(
        attr.span(),
        "connection must be specified as an assignment expression";
        help = "For example, #[connection = \"Conn\"]";
    )
}
