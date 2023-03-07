pub mod acid_repo;
pub mod repository;

use crate::ALLOWED_CLIENTS;
use std::collections::HashMap;
use syn::{
    punctuated::Punctuated,
    spanned::Spanned,
    token::{Add, Comma},
    GenericParam, Generics, TypeParamBound,
};

/// Modifies the generic connection type to the concrete implementation
/// given by `conn_replace` and returns the replaced trait bounds and their indices.
///
/// This function is used to remove the generic parameter from the arguments directly following
/// `impl`.
fn modify_type_generics(
    generics: &mut Generics,

    replacements: &HashMap<String, String>,
) -> Vec<(usize, GenericParam)> {
    let keys = replacements.keys().collect::<Vec<_>>();

    let mut replaced = vec![];

    let types = generics
        .type_params_mut()
        .enumerate()
        .filter_map(|(i, param)| {
            // For the initial impl scoping we want to exclude the generic connection
            // as it will be concrete
            if !keys.contains(&&param.ident.to_string()) {
                // We also need to concretise any bound if they are not in the where clause
                concretise_bounds(&mut param.bounds, replacements);
                Some(GenericParam::Type(param.clone()))
            } else {
                // And we want to concretise those bounds in the where clause
                // so we clone and keep track of where in the impl we concretised
                let mut param = param.clone();
                let replace = replacements.get(&param.ident.to_string()).unwrap();
                let replace = match replace.as_str() {
                    "mongo" => "ClientSession",
                    "postgres" => "PgPoolConnection",
                    _ => unreachable!(),
                };
                param.ident = syn::Ident::new(replace, param.span());
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
fn modify_where_clause(generics: &mut Generics, replacements: &HashMap<String, String>) {
    let where_predicates = &mut generics.make_where_clause().predicates;

    for pred in where_predicates.iter_mut() {
        let syn::WherePredicate::Type(_ty) = pred else { continue; };
        concretise_bounds(&mut _ty.bounds, replacements);
    }
}

/// Replace `target_conn_bound` generic bounds with the concrete one
fn concretise_bounds(
    bounds: &mut Punctuated<TypeParamBound, Add>,
    replacements: &HashMap<String, String>,
) {
    for bound in bounds.iter_mut() {
        // We care only about trait bounds
        let syn::TypeParamBound::Trait(ref mut trait_bound) = bound else { continue; };

        for seg in trait_bound.path.segments.iter_mut() {
            // Concretises `T: Connection`
            if let Some(target) = replacements.get(&seg.ident.to_string()) {
                match target.as_str() {
                    "mongo" => seg.ident = syn::Ident::new("ClientSession", seg.span()),
                    "postgres" => seg.ident = syn::Ident::new("PgPoolConnection", seg.span()),
                    _ => unreachable!(),
                }
            }

            let syn::PathArguments::AngleBracketed(ref mut ab_args) = seg.arguments else { continue; };

            for arg in ab_args.args.iter_mut() {
                match arg {
                    // Concretises the bound argoument `T: Something<Connection>`
                    syn::GenericArgument::Type(ref mut ty) => match ty {
                        syn::Type::Path(ref mut p) => {
                            for seg in p.path.segments.iter_mut() {
                                if let Some(target) = replacements.get(&seg.ident.to_string()) {
                                    match target.as_str() {
                                        "mongo" => {
                                            seg.ident = syn::Ident::new("ClientSession", seg.span())
                                        }
                                        "postgres" => {
                                            seg.ident =
                                                syn::Ident::new("PgPoolConnection", seg.span())
                                        }
                                        _ => unreachable!(),
                                    }
                                }
                            }
                        }
                        _ => {}
                    },
                    // Concretises any connection in associated types, eg `T<Something = Connection>`
                    syn::GenericArgument::Binding(ref mut bind) => match bind.ty {
                        syn::Type::Path(ref mut path) => {
                            for seg in path.path.segments.iter_mut() {
                                if let Some(target) = replacements.get(&seg.ident.to_string()) {
                                    match target.as_str() {
                                        "mongo" => {
                                            seg.ident = syn::Ident::new("ClientSession", seg.span())
                                        }
                                        "postgres" => {
                                            seg.ident =
                                                syn::Ident::new("PgPoolConnection", seg.span())
                                        }
                                        _ => unreachable!(),
                                    }
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

fn extract_conn_attrs(ast: &syn::DeriveInput) -> syn::Result<HashMap<String, String>> {
    let attributes = ast
        .attrs
        .iter()
        .filter_map(|a| {
            let ident_token = a.tokens.clone().into_iter().last();
            ident_token.and_then(|token| {
                for name in ALLOWED_CLIENTS {
                    if a.path.is_ident(name) {
                        let token = token.to_string().replace("(", "").replace(")", "");
                        return Some((token, name.to_string()));
                    }
                }
                None
            })
        })
        .collect::<HashMap<String, String>>();
    Ok(attributes)
}

fn has_connections(map: &HashMap<String, String>) -> (bool, bool) {
    let inverse = map
        .clone()
        .into_iter()
        .map(|(key, val)| (val, key))
        .collect::<HashMap<_, _>>();
    (
        inverse.contains_key(ALLOWED_CLIENTS[0]),
        inverse.contains_key(ALLOWED_CLIENTS[1]),
    )
}
