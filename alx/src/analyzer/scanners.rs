use crate::{
    analyzer::util::{analyze_call_recursive, analyze_path_recursive},
    config::{Data, Field, Handler, HandlerInput, Route},
    print,
};
use colored::Colorize;
use std::collections::HashMap;
use std::fmt::Write;

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

    print(&format!(
        "Found {} inner cfg.service calls",
        inner_calls.len()
    ));

    let mut setup = Vec::<Route>::new();

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

                        analyze_call_recursive(service_call.clone(), &mut route_config, &mut level);

                        // Entry at key 100 will always contain the route path
                        let mut path = route_config.get(&100).unwrap().clone();
                        let path = path.pop().unwrap();

                        // Create temporary vec in case of multiple middleware wraps
                        let mut temp_routes: Vec<Route> = vec![];
                        let mut index = 0;
                        while let Some(item) = route_config.get(&index) {
                            let mut mw = vec![];
                            let j = check_next(&route_config, &mut index, &mut mw);
                            if j == index + 1 {
                                // Since middleware wraps everything before it we have to add it
                                // to the preceding routes
                                if !mw.is_empty() {
                                    temp_routes.iter_mut().for_each(|route| {
                                        if let Some(ref mut mdlw) = route.middleware {
                                            for m in mw.iter() {
                                                mdlw.push(m.clone())
                                            }
                                        } else {
                                            route.middleware = Some(mw.clone());
                                        }
                                    });
                                }
                                let route = Route {
                                    method: item[0].to_string(),
                                    handler_name: item[2].to_string(),
                                    path: path.clone(),
                                    middleware: if mw.is_empty() {
                                        None
                                    } else {
                                        Some(mw.clone())
                                    },
                                    service: if item[1].is_empty() {
                                        None
                                    } else {
                                        Some(item[1].to_string())
                                    },
                                };
                                temp_routes.push(route);
                                index += 1;
                                mw.clear();
                            } else {
                                index = j;
                            }
                        }
                        for tr in temp_routes {
                            setup.push(tr);
                        }
                    }
                }
            }
        }
    }
    setup
}

/// Check if the next hashmap entry is a middleware and call this recursively
/// until we find a route entry. Populates the given mw vector with the found
/// middleware. Returns the index of the first found route entry if any.
fn check_next(
    hm: &HashMap<usize, Vec<String>>,
    current: &mut usize,
    mw: &mut Vec<String>,
) -> usize {
    match hm.get(&(*current + 1)) {
        Some(entry) if entry.len() == 1 => {
            mw.push(entry[0].clone());
            *current += 1;
            check_next(hm, current, mw)
        }
        _ => *current + 1,
    }
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
                            // The identity is the extractor type which holds the inputs type,
                            // usually in angle bracket argument form
                            let ext_type = seg.ident.to_string();
                            let data_type = match seg.arguments {
                                syn::PathArguments::AngleBracketed(arg) => {
                                    // There's usually just one angle bracketed arg since all the data
                                    // should come from some kind of wrapper struct from data.rs
                                    match arg.args.first().unwrap() {
                                        syn::GenericArgument::Type(ty) => {
                                            // The same goes for the path, this is where we'll find the
                                            // data type
                                            if let syn::Type::Path(path) = ty {
                                                path.path
                                                    .segments
                                                    .first()
                                                    .unwrap()
                                                    .ident
                                                    .to_string()
                                                    .replace("Payload", "")
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
        print(&format!("👌 Found handler {}", handler.name));
        handlers.push(handler);
    }
    handlers
}

/// Scan the data.rs file for data info
pub(super) fn scan_data(items: Vec<syn::Item>) -> Vec<Data> {
    let mut inputs = vec![];
    for item in items.iter() {
        let mut input = Data::default();

        // We filtered struct items before sending them to this functions
        // so they will all be structs
        if let syn::Item::Struct(strct) = item {
            if strct.ident.to_string().contains("Response") {
                continue;
            }
            input.id = strct.ident.to_string();
            // Iterate through struct fields
            for field in strct.fields.iter() {
                let name = field.ident.as_ref().unwrap().to_string();
                let mut f = Field {
                    name,
                    ty: String::new(),
                    required: false,
                    validation: vec![],
                };

                // Struct fields will always be Path types
                if let syn::Type::Path(ref ty) = field.ty {
                    let mut nested: Vec<String> = vec![];
                    analyze_path_recursive(ty, &mut f, &mut nested);
                    if !nested.is_empty() {
                        let mut typ = nested.join("<");
                        write!(typ, "{}", ">".repeat(nested.len() - 1)).unwrap();
                        if typ.contains("Option") {
                            f.required = false;
                        }
                        f.ty = typ;
                    }
                }

                // And search for field attributes for validation
                for attr in field.attrs.iter() {
                    let validation = attr.path.is_ident("validate");
                    if validation {
                        let val = attr
                            .tokens
                            .to_string()
                            .replacen('(', "", 1)
                            .replacen(')', "", 1);
                        f.validation.push(val);
                    }
                }
                input.fields.push(f);
            }
        }
        print(&format!("💽 Found request data {}", input.id));
        inputs.push(input);
    }
    inputs
}
