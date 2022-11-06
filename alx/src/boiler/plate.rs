use crate::{uppercase,  INDENT, MW_FILES, ROUTE_FILES};
use std::fmt::Write;

pub enum BoilerType {
    Route,
    MW,
}

pub fn r#mod(buf: &mut String, ty: BoilerType) {
    match ty {
        BoilerType::Route => {
            for f in ROUTE_FILES {
                if f != "mod" {
                    if f == "setup" {
                        writeln!(buf, "pub(crate) mod {};", f).unwrap();
                    } else {
                        writeln!(buf, "pub(super) mod {};", f).unwrap();
                    }
                }
            }
            write!(buf, "\n#[cfg(test)]\nmod tests {{\n\n{INDENT}#[test]\n{INDENT}fn test() {{\n{INDENT}{INDENT}assert!(true)\n{INDENT}}}\n}}").unwrap();
        }
        BoilerType::MW => {
            for f in MW_FILES {
                if f != "mod" {
                    if f == "interceptor" {
                        writeln!(buf, "pub(crate) mod {};", f).unwrap();
                    } else {
                        writeln!(buf, "pub(super) mod {};", f).unwrap();
                    }
                }
            }
            write!(buf, "\n#[cfg(test)]\nmod tests {{\n\n{INDENT}#[test]\n{INDENT}fn test() {{\n{INDENT}{INDENT}assert!(true)\n{INDENT}}}\n}}").unwrap();
        }
    }
}

pub fn contracts(buf: &mut String, contracts: &[&str], ty: BoilerType) {
    let vis = match ty {
        BoilerType::Route => "super",
        BoilerType::MW => "crate",
    };
    writeln!(buf, "use async_trait::async_trait;\n").unwrap();
    writeln!(buf, "#[cfg_attr(test, mockall::automock)]\n#[async_trait]\npub({vis}) trait ServiceContract {{}}").unwrap();
    for c in contracts {
        writeln!(
            buf,
            "\n#[cfg_attr(test, mockall::automock)]\n#[async_trait]\npub({vis}) trait {}Contract {{}}",
            uppercase(c)
        ).unwrap();
    }
}

pub fn infrastructure(buf: &mut String, contracts: &[&str]) {
    let mut use_stmt = String::new();
    let mut struct_stmt = String::new();
    write!(use_stmt, "use super::contract::").unwrap();
    if contracts.len() == 1 {
        writeln!(use_stmt, "{}Contract;", uppercase(contracts[0])).unwrap();
        writeln!(
            struct_stmt,
            "pub(super) struct {} {{}}\n\n#[async_trait]\nimpl {}Contract for {} {{}}",
            uppercase(contracts[0]),
            uppercase(contracts[0]),
            uppercase(contracts[0]),
        )
        .unwrap();
    } else {
        write!(use_stmt, "{{").unwrap();
        for (i, c) in contracts.iter().enumerate() {
            if i == contracts.len() - 1 {
                writeln!(use_stmt, "{}Contract}};", uppercase(c)).unwrap();
            } else {
                write!(use_stmt, "{}Contract, ", uppercase(c)).unwrap();
            }
            writeln!(
                struct_stmt,
                "pub(super) struct {} {{}}\n\n#[async_trait]\nimpl {}Contract for {} {{}}\n",
                uppercase(c),
                uppercase(c),
                uppercase(c),
            )
            .unwrap();
        }
    }
    writeln!(use_stmt, "use async_trait::async_trait;\n").unwrap();
    write!(buf, "{}{}", use_stmt, struct_stmt).unwrap();
}

pub fn setup(buf: &mut String, service_name: &str, contracts: &[&str]) {
    // Use statement
    let mut use_stmt = format!(
        "use super::{{\n{INDENT}domain::{},\n{INDENT}handler,\n",
        service_name
    );
    if !contracts.is_empty() {
        if contracts.len() == 1 {
            writeln!(
                use_stmt,
                "{INDENT}infrastructure::{},\n}};",
                uppercase(contracts[0])
            )
            .unwrap();
        } else {
            write!(use_stmt, "{INDENT}infrastructure::{{").unwrap();
            for (i, c) in contracts.iter().enumerate() {
                if i == contracts.len() - 1 {
                    write!(use_stmt, "{}", &uppercase(c)).unwrap();
                } else {
                    write!(use_stmt, "{}, ", &uppercase(c)).unwrap();
                }
            }
            writeln!(use_stmt, "}},\n}};").unwrap();
        }
    } else {
        writeln!(use_stmt, "}};").unwrap();
    }
    write!(use_stmt, "use actix_web::web;\n\n").unwrap();

    let fn_stmt = String::from("pub(crate) fn routes(cfg: &mut web::ServiceConfig) {}");

    write!(buf, "{}{}", use_stmt, fn_stmt).unwrap();
}

pub fn domain(buf: &mut String, service_name: &str, contracts: &[&str]) {
    // Use statement
    let mut use_stmt = String::from("use super::contract::");
    if !contracts.is_empty() {
        write!(use_stmt, "{{ServiceContract, ").unwrap();
        write_contracts_use(&mut use_stmt, contracts);
    } else {
        writeln!(use_stmt, "ServiceContract;").unwrap();
    }
    writeln!(use_stmt, "use async_trait::async_trait;\n").unwrap();

    // Struct statement
    let mut struct_statement = format!("#[derive(Debug)]\npub(super) struct {}", service_name);
    if !contracts.is_empty() {
        write_letter_bounds(&mut struct_statement, contracts, &[]);
        writeln!(struct_statement, "\nwhere").unwrap();
        for c in contracts {
            writeln!(
                struct_statement,
                "{INDENT}{}: {}Contract,",
                &uppercase(c)[..1],
                uppercase(c)
            )
            .unwrap();
        }
        writeln!(struct_statement, "{{").unwrap();
        for c in contracts {
            writeln!(
                struct_statement,
                "{INDENT}pub {}: {},",
                c,
                &uppercase(c)[..1],
            )
            .unwrap();
        }
        writeln!(struct_statement, "}}\n").unwrap();
    } else {
        writeln!(struct_statement, ";\n").unwrap();
    }

    // Impl statement
    let mut impl_stmt = String::from("#[async_trait]\nimpl");
    if !contracts.is_empty() {
        write_letter_bounds(&mut impl_stmt, contracts, &[]);
    }
    write!(impl_stmt, " ServiceContract for {}", service_name).unwrap();
    if !contracts.is_empty() {
        write_letter_bounds(&mut impl_stmt, contracts, &[]);
        writeln!(impl_stmt, "\nwhere").unwrap();
        for c in contracts {
            writeln!(
                impl_stmt,
                "{INDENT}{}: {}Contract + Send + Sync,",
                &uppercase(c)[..1],
                uppercase(c)
            )
            .unwrap();
        }
        write!(impl_stmt, "{{\n}}").unwrap();
    } else {
        write!(impl_stmt, " {{}}").unwrap();
    }

    writeln!(buf, "{}{}{}", use_stmt, struct_statement, impl_stmt).unwrap();
}

pub fn mw_interceptor(buf: &mut String, service_name: &str, contracts: &[&str]) {
    /*
     * Use statement
     */
    let mut use_stmt = String::new();
    write!(use_stmt, "use super::contract::").unwrap();
    if !contracts.is_empty() {
        write!(use_stmt, "{{ServiceContract, ",).unwrap();
        write_contracts_use(&mut use_stmt, contracts);
    } else {
        writeln!(use_stmt, "ServiceContract;").unwrap();
    }
    writeln!(use_stmt, "use super::domain::{};", service_name).unwrap();
    writeln!(use_stmt, "use actix_web::dev::{{forward_ready, Service, ServiceRequest, ServiceResponse, Transform}};").unwrap();
    writeln!(
        use_stmt,
        "use futures_util::future::LocalBoxFuture;\nuse futures_util::FutureExt;"
    )
    .unwrap();
    writeln!(
        use_stmt,
        "use std::{{\n{INDENT}future::{{ready, Ready}},\n{INDENT}rc::Rc,\n}};\n"
    )
    .unwrap();

    /*
     * Struct statement
     */
    let mut struct_stmt = String::new();
    writeln!(struct_stmt, "#[derive(Debug, Clone)]").unwrap();
    write!(struct_stmt, "pub(crate) struct {}Guard", service_name).unwrap();
    if !contracts.is_empty() {
        write_letter_bounds(&mut struct_stmt, contracts, &[]);
        writeln!(struct_stmt, "\nwhere").unwrap();
        for c in contracts {
            writeln!(
                struct_stmt,
                "{INDENT}{}: {}Contract,",
                &uppercase(c)[..1],
                uppercase(c)
            )
            .unwrap();
        }
    }
    if !contracts.is_empty() {
        write!(struct_stmt, "{{\n{INDENT}guard: Rc<{}", service_name).unwrap();
    } else {
        write!(struct_stmt, " {{\n{INDENT}guard: Rc<{}", service_name).unwrap();
    }
    if !contracts.is_empty() {
        write_letter_bounds(&mut struct_stmt, contracts, &[]);
    }
    writeln!(struct_stmt, ">,\n}}\n").unwrap();

    /*
     * Impl Transform statement
     */
    let mut transform_impl = String::from("impl");
    write_letter_bounds(&mut transform_impl, contracts, &["S"]);
    write!(transform_impl, " Transform<S, ServiceRequest> for {}Guard", service_name).unwrap();
    if !contracts.is_empty() {
        write_letter_bounds(&mut transform_impl, contracts, &[]);
    }
    writeln!(transform_impl, "\nwhere").unwrap();
    writeln!(transform_impl, "{INDENT}S: Service<ServiceRequest, Response = ServiceResponse, Error = actix_web::Error> + 'static,").unwrap();
    writeln!(transform_impl, "{INDENT}S::Future: 'static,").unwrap();
    for c in contracts {
        writeln!(transform_impl, "{INDENT}{}: {}Contract + Send + Sync + 'static,", &uppercase(c)[..1], &uppercase(c)).unwrap();
    }
    writeln!(transform_impl, "{{").unwrap();
    write!(
        transform_impl, 
        "{INDENT}type Response = ServiceResponse;\n{INDENT}type Error = actix_web::Error;\n{INDENT}type InitError = ();\n{INDENT}type Transform = {}Middleware",
        service_name,
    ).unwrap();
    write_letter_bounds(&mut transform_impl, contracts, &["S"]);
    writeln!(transform_impl, ";\n{INDENT}type Future = Ready<Result<Self::Transform, Self::InitError>>;\n").unwrap();
    writeln!(transform_impl, "{INDENT}fn new_transform(&self, service: S) -> Self::Future {{").unwrap();
    writeln!(transform_impl, "{INDENT}{INDENT}ready(Ok({}Middleware {{", service_name).unwrap();
    writeln!(transform_impl, "{INDENT}{INDENT}{INDENT}service: Rc::new(service),").unwrap();
    writeln!(transform_impl, "{INDENT}{INDENT}{INDENT}guard: self.guard.clone(),").unwrap();
    writeln!(transform_impl, "{INDENT}{INDENT}}}))").unwrap();
    writeln!(transform_impl, "{INDENT}}}\n}}\n").unwrap();

    /*
     * Middleware struct statement 
     */
    let mut mw_strct_stmt = format!("pub(crate) struct {}Middleware", service_name);
    write_letter_bounds(&mut mw_strct_stmt, contracts, &["S"]);
    if !contracts.is_empty() {
        writeln!(mw_strct_stmt,"\nwhere").unwrap();
        for c in contracts {
            writeln!(
                mw_strct_stmt,
                "{INDENT}{}: {}Contract,",
                &uppercase(c)[..1],
                uppercase(c)
            ).unwrap();
        }
    }
    if !contracts.is_empty() {
        write!(mw_strct_stmt, "{{\n{INDENT}guard: Rc<{}", service_name).unwrap();
    } else {
        write!(mw_strct_stmt, " {{\n{INDENT}guard: Rc<{}", service_name).unwrap();
    }
    if !contracts.is_empty() {
        write_letter_bounds(&mut mw_strct_stmt, contracts, &[]);
    }
    writeln!(mw_strct_stmt, ">,\n{INDENT}service: Rc<S>,\n}}\n").unwrap();

    /*
     * Impl Service statement 
     */
    let mut service_impl = String::from("impl");
    write_letter_bounds(&mut service_impl, contracts, &["S"]);
    write!(service_impl," Service<ServiceRequest> for {}Middleware", service_name).unwrap();
    write_letter_bounds(&mut service_impl, contracts, &["S"]);
    writeln!(service_impl,"\nwhere").unwrap();
    writeln!(service_impl,"{INDENT}S: Service<ServiceRequest, Response = ServiceResponse, Error = actix_web::Error> + 'static,").unwrap();
    writeln!(service_impl,"{INDENT}S::Future: 'static,").unwrap();
    if !contracts.is_empty() {
        for c in contracts {
            writeln!(
                service_impl,
                "{INDENT}{}: {}Contract + Send + Sync + 'static,",
                &uppercase(c)[..1],
                uppercase(c)
            )
            .unwrap();
        }
    }
    writeln!(service_impl, "{{").unwrap();
    writeln!(service_impl, "{INDENT}type Response = ServiceResponse;").unwrap();
    writeln!(service_impl, "{INDENT}type Error = actix_web::Error;").unwrap();
    writeln!(service_impl, "{INDENT}type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;\n").unwrap();
    writeln!(service_impl, "{INDENT}forward_ready!(service);\n").unwrap();
    writeln!(service_impl, "{INDENT}fn call(&self, req: ServiceRequest) -> Self::Future {{").unwrap();
    writeln!(service_impl, "{INDENT}{INDENT}let guard = self.guard.clone();").unwrap();
    writeln!(service_impl, "{INDENT}{INDENT}let service = self.service.clone();\n").unwrap();
    writeln!(service_impl, "{INDENT}{INDENT}async move {{").unwrap();
    writeln!(service_impl, "{INDENT}{INDENT}{INDENT}let res = service.call(req).await?;\n{INDENT}{INDENT}{INDENT}Ok(res)").unwrap();
    writeln!(service_impl, "{INDENT}{INDENT}}}\n{INDENT}{INDENT}.boxed_local()\n{INDENT}}}\n}}").unwrap();

    write!(buf, "{}{}{}{}{}", use_stmt, struct_stmt, transform_impl, mw_strct_stmt, service_impl).unwrap();
}

fn write_letter_bounds(stmt: &mut String, contracts: &[&str], append: &[&str]) {
    write!(stmt, "<").unwrap();
    for (i, a) in append.iter().enumerate() {
        if contracts.is_empty() {
            if i == append.len() -1  {
                write!(stmt, "{a}").unwrap();
            } else {
                write!(stmt, "{a}, ").unwrap();
            }
        } else {
            write!(stmt, "{a}, ").unwrap();
        }
    }
    for (i, c) in contracts.iter().enumerate() {
        if i == contracts.len() - 1 {
            write!(stmt, "{}", &uppercase(c)[..1]).unwrap();
        } else {
            write!(stmt, "{}, ", &uppercase(c)[..1]).unwrap();
        }
    }
    write!(stmt, ">").unwrap();
}

/// Writes the closing `};` tokens as well
fn write_contracts_use(buf: &mut String, contracts: &[&str]) {
    for (i, c) in contracts.iter().enumerate() {
        if i == contracts.len() - 1 {
            write!(buf, "{}Contract", uppercase(c)).unwrap();
        } else {
            write!(buf, "{}Contract, ", uppercase(c)).unwrap();
        }
    }
    writeln!(buf, "}};").unwrap();
}
