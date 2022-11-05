use crate::{
    analyzer::util::scan_call_recursive,
    config::{Data, Field, Handler, HandlerInput, Route},
};
use colored::Colorize;
use std::collections::HashMap;

/// Scan a setup.rs file for route info
pub(super) fn scan_setup(functions: Vec<syn::ItemFn>) -> Vec<Route> {
    // Get all the statements from its block. This will include all
    // the service initializations and cfg.service() calls
    let mut routes_fn_inner = match functions.first() {
        Some(calls) => calls.block.stmts.clone(),
        None => vec![],
    };

    // Filter out the cfg.service calls
    let inner_calls = routes_fn_inner
        .drain(..)
        .filter(|stmt| matches!(stmt, syn::Stmt::Semi(_, _)))
        .collect::<Vec<syn::Stmt>>();

    println!("Found {} inner cfg.service calls", inner_calls.len());

    let mut setup = Vec::<Route>::new();
    let mut route = Route::default();

    for call in inner_calls {
        // cfg.service() calls will always be method calls
        if let syn::Stmt::Semi(syn::Expr::MethodCall(cfg_service_call), _) = call {
            // The target should always be either the cfg or a *scope -- *todo
            if let syn::Expr::Path(ref service_path) = *cfg_service_call.receiver {
                let target = &service_path
                    .path
                    .segments
                    .first()
                    .unwrap()
                    .ident
                    .to_string();

                if target == "cfg" && cfg_service_call.method == "service" {
                    // These calls always have one argument
                    let arg = cfg_service_call.args.first().unwrap();

                    // And it's probably going to have nested route calls
                    // inside, so we check for those. If the need arises we'll
                    // check for regular expr calls and expr paths as well.
                    if let syn::Expr::MethodCall(ref service_call) = arg {
                        let mut level = 0;
                        let mut route_config = HashMap::<usize, Vec<String>>::new();
                        scan_call_recursive(
                            service_call.clone(),
                            &mut route,
                            &mut level,
                            &mut route_config,
                        );
                        println!("Mapped: {:?}", route_config);
                    }
                    // println!("ARG: {:#?}", arg);
                }
            }
        }
        if route != Route::default() {
            setup.push(route);
            route = Route::default();
        }
    }
    setup
}

/// Scan the handler.rs file for handler info
pub(super) fn scan_handlers(functions: Vec<syn::ItemFn>) -> Vec<Handler> {
    let mut handlers = Vec::<Handler>::new();

    for hand in functions {
        // Grab the name of the handler
        let name = hand.sig.ident.to_string();

        // Check if it has any bounds
        let bound = match hand.sig.generics.params.first() {
            Some(param) => match param {
                // For now only check type bounds since it's
                // unlikely handlers will have lifetime or const bounds
                syn::GenericParam::Type(ty) => {
                    let mut typ = ty.ident.to_string();
                    if let Some(bound) = ty.bounds.first() {
                        match bound {
                            syn::TypeParamBound::Trait(tb) => {
                                typ = format!(
                                    "{}: {}",
                                    typ,
                                    tb.path.segments.first().unwrap().ident,
                                );
                                Some(typ)
                            }
                            syn::TypeParamBound::Lifetime(_) => todo!(),
                        }
                    } else {
                        Some(typ)
                    }
                }
                syn::GenericParam::Lifetime(_) => todo!(),
                syn::GenericParam::Const(_) => todo!(),
            },
            None => None,
        };

        let mut handler = Handler {
            name,
            inputs: vec![],
            bound,
        };

        hand.sig.inputs.into_iter().for_each(|fn_arg| match fn_arg {
            // Skip references to self in handlers
            syn::FnArg::Receiver(f) => {
                println!("{} {:?}", "Found self in handler?".red(), f)
            }
            // And iterate through all the args of the function
            syn::FnArg::Typed(args) => {
                if let syn::Type::Path(p) = *args.ty {
                    // Iterate through all the type params
                    for seg in p.path.segments {
                        // We don't care about the web prefix
                        if seg.ident != "web" {
                            // The identity is the extractor type which holds the data type,
                            // usually in angle bracket argument form
                            let ext_type = seg.ident.to_string();
                            let data_type = match seg.arguments {
                                syn::PathArguments::AngleBracketed(arg) => {
                                    // There's usually just one angle bracketed arg since all the data
                                    // should come from some kind of wrapper struct from data.rs
                                    match arg.args.first().unwrap() {
                                        syn::GenericArgument::Type(t) => {
                                            // The same goes for the path, this is where we'll find the
                                            // data type
                                            if let syn::Type::Path(t) = t {
                                                t.path.segments.first().unwrap().ident.to_string()
                                            } else {
                                                panic!("Found a funky syn Type")
                                            }
                                        }
                                        _ => panic!("Found some funky angle bracket arguments"),
                                    }
                                }
                                syn::PathArguments::Parenthesized(_) => String::new(),
                                syn::PathArguments::None => String::new(),
                            };
                            handler.inputs.push(HandlerInput {
                                ext_type,
                                data_type,
                            })
                        }
                    }
                }
            }
        });
        println!("Created {:?}", handler);
        handlers.push(handler);
    }
    handlers
}

pub(super) fn scan_data(items: Vec<syn::Item>) -> Vec<Data> {
    let data = vec![];
    for item in items.iter() {
        println!("ITEM: {:#?}", item);
        let mut data = Data::default();
        // We filtered struct items before sending them to this functions
        // so they will all be structs
        if let syn::Item::Struct(strct) = item {
            data.wrapper_id = strct.ident.to_string();
            println!("STRUCT ID: {}", data.wrapper_id);
            for field in strct.fields.iter() {
                let mut f = Field::default();
                println!("FIELD IDENT: {}", field.ident.as_ref().unwrap().to_string());
                f.name = field.ident.as_ref().unwrap().to_string();

                if let syn::Type::Path(ref ty) = field.ty {
                    if let Some(ref id) = ty.path.get_ident() {
                        println!("IDENT IS SIMPLE: {:#?}", ty);
                        f.required = true;
                        f.ty = id.to_string();
                    } else {
                        println!("IDENT IS MORE COMPLEX: {:#?}", ty);
                    }
                }

                for attr in field.attrs.iter() {
                    let validation = attr.path.is_ident("validate");
                    println!("HAS VALIDATION: {}", validation);
                    if validation {}
                    println!("VALIDATION TOKENS: {}", attr.tokens.to_string());
                }
                println!("FIELD: {:#?}", f);
            }
        }
    }
    data
}
