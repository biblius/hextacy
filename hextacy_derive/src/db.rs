pub mod adapter;

use lazy_static::lazy_static;
use proc_macro2::Span;
use proc_macro_error::abort;
use std::collections::HashMap;
use syn::{
    punctuated::Punctuated, spanned::Spanned, token::Comma, GenericParam, Generics, Ident, Token,
    TypeParamBound,
};

const DIESEL_CONNECTION: &str = "PgPoolConnection";
const SEAORM_CONNECTION: &str = "DatabaseConnection";
const MONGO_CONNECTION: &str = "ClientSession";
const DRIVERS: [&str; 3] = ["diesel", "mongo", "seaorm"];
const _CONNECTIONS: [&str; 3] = ["PgPoolConnection", "ClientSession", "DatabaseConnection"];

lazy_static! {
    pub static ref CONNECTIONS: HashMap<&'static str, &'static str> = {
        DRIVERS
            .iter()
            .zip(_CONNECTIONS)
            .map(|(driver, connection)| (*driver, connection))
            .collect()
    };
}

/// Modifies the generics in a way that excludes generics bounds from the initial impl scoping
/// and concretises bounds in the where clause to the concrete connections.
fn process_generics(
    ast: &mut syn::DeriveInput,
    replacements: &HashMap<String, Ident>,
) -> (Vec<(usize, GenericParam)>, Generics) {
    // Clone the generics as we do not want to modify the input
    let mut generics = ast.generics.clone();

    // Modify the type generics by taking out the specified connection generic from the impl and substitute
    // it in the following bounds with whatever they are paired with
    let replaced = modify_type_generics(&mut generics, replacements);
    if replaced.is_empty() {
        abort!(
            ast.ident.span(),
            format!("Repository derive needs at least one driver field");
            help = "Annotate the Driver field with one of the available drivers: diesel, mongo, seaorm"
        );
    }

    (replaced, generics)
}

/// Modifies the generic connection type to the concrete implementation
/// given by `conn_replace` and returns the replaced trait bounds and their indices.
///
/// This function is used to remove the generic parameter from the arguments directly following
/// `impl`.
fn modify_type_generics(
    generics: &mut Generics,
    replacements: &HashMap<String, Ident>,
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
                param.ident = replace.clone();
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
fn modify_where_clause(generics: &mut Generics, replacements: &HashMap<String, Ident>) {
    let where_predicates = &mut generics.make_where_clause().predicates;

    for pred in where_predicates.iter_mut() {
        let syn::WherePredicate::Type(_ty) = pred else { continue; };
        concretise_bounds(&mut _ty.bounds, replacements);
    }
}

/// Replace `target_conn_bound` generic bounds with the concrete one
fn concretise_bounds(
    bounds: &mut Punctuated<TypeParamBound, Token!(+)>,
    replacements: &HashMap<String, Ident>,
) {
    for bound in bounds.iter_mut() {
        // We care only about trait bounds
        let syn::TypeParamBound::Trait(ref mut trait_bound) = bound else { continue; };

        for seg in trait_bound.path.segments.iter_mut() {
            // Concretises `T: Connection`
            if let Some(target) = replacements.get(&seg.ident.to_string()) {
                seg.ident = target.clone();
            }

            let syn::PathArguments::AngleBracketed(ref mut ab_args) = seg.arguments else { continue; };

            for arg in ab_args.args.iter_mut() {
                match arg {
                    // Concretises the bound argoument `T: Something<Connection>`
                    syn::GenericArgument::Type(ref mut ty) => {
                        let syn::Type::Path(ref mut p) = ty else {
                            continue;
                        };
                        for seg in p.path.segments.iter_mut() {
                            if let Some(target) = replacements.get(&seg.ident.to_string()) {
                                seg.ident = target.clone();
                            }
                        }
                    }
                    // Concretises any connection in associated types, eg `T<Something = Connection>`
                    syn::GenericArgument::AssocType(ref mut bind) => {
                        let syn::Type::Path(ref mut path) = bind.ty else {
                            continue;
                        };

                        for seg in path.path.segments.iter_mut() {
                            if let Some(target) = replacements.get(&seg.ident.to_string()) {
                                seg.ident = target.clone();
                            }
                        }
                    }
                    // This should theoretically be unreachable
                    syn::GenericArgument::Const(_)
                    | syn::GenericArgument::Lifetime(_)
                    | syn::GenericArgument::Constraint(_)
                    | syn::GenericArgument::AssocConst(_) => unimplemented!(),
                    _ => todo!(),
                }
            }
        }
    }
}

fn scan_fields(ast: &syn::DeriveInput) -> Fields {
    match ast.data {
        syn::Data::Struct(ref strct) => {
            let mut fields = Fields {
                driver_fields: HashMap::new(),
                replacements: HashMap::new(),
            };

            for field in strct.fields.iter() {
                for attr in field.attrs.iter() {
                    for seg in attr.path().segments.iter() {
                        if let Some(concretised) = CONNECTIONS.get(seg.ident.to_string().as_str()) {
                            let generic = attr
                                .meta
                                .require_list()
                                .unwrap_or_else(|_| {
                                    abort!(attr.span(), "Invalid attribute given for driver")
                                })
                                .tokens
                                .to_string()
                                .replace(['(', ')'], "");
                            fields
                                .replacements
                                .insert(generic, Ident::new(concretised, Span::call_site()));
                            fields.driver_fields.insert(
                                concretised.to_string(),
                                field.ident.clone().expect("Ident required"),
                            );
                        }
                    }
                }
            }
            fields
        }
        _ => {
            abort!(
                ast.span(),
                "Driver and transaction attributes can only be used on field structs"
            )
        }
    }
}

#[derive(Debug)]
struct Fields {
    driver_fields: HashMap<String, Ident>,
    replacements: HashMap<String, Ident>,
}
