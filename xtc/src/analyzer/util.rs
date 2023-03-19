use crate::config::Field;
use std::collections::HashMap;
use syn::{ExprMethodCall, TypePath};

/// Read a method call recursively and map all its arguments and receivers. The route_config values after
/// this will always be in the following order: `["METHOD", "SERVICE", "HANDLER"]`, excluding the 0th
/// value which will have the path as the 0th element and the rest as described (4 in total). The way
/// the setup file is written is such that the arguments will almost always be method calls so this function
/// is mostly concerned with that. The method calls will most likely be nested if the cfg.service() call has
/// more than one route or is wrapped by a middleware. If the need arises this function will have to be refined
/// but for now it works pretty good. If a route is mapped at some key and if the next value's length is 1, we can
/// almost be certain that value is a name, since middleware is usually parameterised as a path with a single element.
pub(super) fn analyze_call_recursive(
    method_call: ExprMethodCall,
    route_config: &mut HashMap<usize, Vec<String>>,
    level: &mut usize,
) {
    // If the receiver is another method call, scan it recursively.
    if let syn::Expr::MethodCall(ref meth_call) = *method_call.receiver {
        analyze_call_recursive(meth_call.clone(), route_config, level);
    }

    // This checks for the web::resource("/path") string literal. Since this will be common to all
    // the service routes we map this to 100.
    if let syn::Expr::Call(ref call) = *method_call.receiver {
        if let Some(syn::Expr::Lit(ref p)) = call.args.first() {
            if let syn::Lit::Str(ref path) = p.lit {
                route_config
                    .entry(100)
                    .and_modify(|e| e.push(path.value()))
                    .or_insert_with(|| vec![path.value()]);
            }
        }
    }

    // Iterate through all the method call arguments
    for mut arg in method_call.args {
        // Middleware wrappers, i.e. some_guard in `.wrap(some_guard)` will be a path argument
        if let syn::Expr::Path(ref path) = arg {
            if let Some(wrapper) = path.path.get_ident() {
                route_config
                    .entry(*level)
                    .and_modify(|e| e.push(wrapper.to_string()))
                    .or_insert_with(|| vec![wrapper.to_string()]);
            }
        }

        // Check for more method calls
        if let syn::Expr::MethodCall(ref mut meth_call) = arg {
            // And if the receiver is another one scan recursively
            if let syn::Expr::MethodCall(ref call) = *meth_call.receiver {
                analyze_call_recursive(call.clone(), route_config, level);
            }

            // Otherwise check if the receiver is a function call
            if let syn::Expr::Call(ref mut call) = *meth_call.receiver {
                // Look for a web::method() call
                if let syn::Expr::Path(ref mut call) = *call.func {
                    let method = &mut call
                        .path
                        .segments
                        .pop()
                        .unwrap()
                        .value()
                        .ident
                        .to_string()
                        .to_uppercase();

                    route_config
                        .entry(*level)
                        .and_modify(|e| e.push(method.clone()))
                        .or_insert_with(|| vec![method.clone()]);
                }
                // Look for a path literal i.e. web::resource("/something")
                if let Some(syn::Expr::Lit(ref p)) = call.args.first() {
                    if let syn::Lit::Str(ref path) = p.lit {
                        route_config
                            .entry(*level)
                            .and_modify(|e| e.push(path.value()))
                            .or_insert_with(|| vec![path.value()]);
                    }
                }
            }

            // We also have to check for wrappers in method call arguments
            if let syn::Expr::Path(ref path) = *meth_call.receiver {
                if let Some(wrapper) = path.path.get_ident() {
                    route_config
                        .entry(*level)
                        .and_modify(|e| e.push(wrapper.to_string()))
                        .or_insert_with(|| vec![wrapper.to_string()]);
                }
            }

            // Get the name of the handler
            if let Some(syn::Expr::Path(route_path)) = meth_call.args.first() {
                let mut service = None;
                let mut handlers = route_path
                    .path
                    .segments
                    .iter()
                    .filter_map(|p| {
                        if p.ident != "handler" {
                            // Get the service associated with the handler if any
                            if let syn::PathArguments::AngleBracketed(ref args) = p.arguments {
                                for arg in &args.args {
                                    if let syn::GenericArgument::Type(syn::Type::Path(p)) = arg {
                                        service = Some(
                                            p.path.segments.first().unwrap().ident.to_string(),
                                        );
                                    }
                                }
                            }
                            return Some(p.ident.to_string());
                        }
                        None
                    })
                    .collect::<Vec<String>>();

                let h = handlers.pop().unwrap();
                let s = service.clone().unwrap_or_default();

                // Insert route_config into the map
                route_config
                    .entry(*level)
                    .and_modify(|e| e.push(s.clone()))
                    .or_insert_with(|| vec![s.clone()]);
                route_config
                    .entry(*level)
                    .and_modify(|e| e.push(h.to_string()))
                    .or_insert_with(|| vec![h.to_string()]);
            }
        }
    }
    *level += 1;
}

pub(super) fn analyze_path_recursive(ty: &TypePath, f: &mut Field, buf: &mut Vec<String>) {
    // First check for a simple identifier, i.e. a field with a single primitive
    if let Some(ref id) = ty.path.get_ident() {
        f.required = true;
        f.ty = id.to_string();
    } else {
        // Otherwise check for angle bracket nested types such as Options and Vecs
        // Grab the top level identifier of the type in question
        let typ = ty.path.segments.first().unwrap().ident.to_string();
        buf.push(typ);

        // Here we can safely unwrap because we know it contains something since we entered the else block
        match ty.path.segments.first().unwrap().arguments {
            // I've yet to see a type that contains params without angle brackets
            syn::PathArguments::AngleBracketed(ref args) => {
                // Check again for a simple type, if it's not simple go through the function once more
                if let syn::GenericArgument::Type(syn::Type::Path(t)) = args.args.first().unwrap() {
                    if let Some(id) = t.path.get_ident() {
                        buf.push(id.to_string());
                    } else {
                        analyze_path_recursive(t, f, buf);
                    }
                }
            }
            _ => panic!("Not yet supported"),
        }
    }
}
