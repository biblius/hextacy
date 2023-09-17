use super::{write_apis_use, write_letter_bounds};
use crate::{uppercase, INDENT};
use std::fmt::Write;

pub fn mw_interceptor(buf: &mut String, service_name: &str, apis: &[&str]) {
    /*
     * Use statement
     */
    let mut use_stmt = String::new();
    write!(use_stmt, "use super::api::").unwrap();
    if !apis.is_empty() {
        write!(use_stmt, "{{ServiceApi, ",).unwrap();
        write_apis_use(&mut use_stmt, apis);
    } else {
        writeln!(use_stmt, "ServiceApi;").unwrap();
    }
    writeln!(use_stmt, "use super::domain::{service_name};").unwrap();
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
    write!(struct_stmt, "pub(crate) struct {service_name}Guard").unwrap();
    if !apis.is_empty() {
        write_letter_bounds(&mut struct_stmt, apis, &[]);
        writeln!(struct_stmt, "\nwhere").unwrap();
        for c in apis {
            writeln!(
                struct_stmt,
                "{INDENT}{}: {}Api,",
                &uppercase(c)[..1],
                uppercase(c)
            )
            .unwrap();
        }
    }
    if !apis.is_empty() {
        write!(struct_stmt, "{{\n{INDENT}guard: Rc<{service_name}").unwrap();
    } else {
        write!(struct_stmt, " {{\n{INDENT}guard: Rc<{service_name}").unwrap();
    }
    if !apis.is_empty() {
        write_letter_bounds(&mut struct_stmt, apis, &[]);
    }
    writeln!(struct_stmt, ">,\n}}\n").unwrap();

    /*
     * Impl Transform statement
     */
    let mut transform_impl = String::from("impl");
    write_letter_bounds(&mut transform_impl, apis, &["S"]);
    write!(
        transform_impl,
        " Transform<S, ServiceRequest> for {service_name}Guard"
    )
    .unwrap();
    if !apis.is_empty() {
        write_letter_bounds(&mut transform_impl, apis, &[]);
    }
    writeln!(transform_impl, "\nwhere").unwrap();
    writeln!(transform_impl, "{INDENT}S: Service<ServiceRequest, Response = ServiceResponse, Error = actix_web::Error> + 'static,").unwrap();
    writeln!(transform_impl, "{INDENT}S::Future: 'static,").unwrap();
    for api in apis {
        writeln!(
            transform_impl,
            "{INDENT}{}: {}Api + Send + Sync + 'static,",
            &uppercase(api)[..1],
            &uppercase(api)
        )
        .unwrap();
    }
    writeln!(transform_impl, "{{").unwrap();
    write!(
        transform_impl,
        "{INDENT}type Response = ServiceResponse;\n{INDENT}type Error = actix_web::Error;\n{INDENT}type InitError = ();\n{INDENT}type Transform = {service_name}Middleware"
    ).unwrap();
    write_letter_bounds(&mut transform_impl, apis, &["S"]);
    writeln!(
        transform_impl,
        ";\n{INDENT}type Future = Ready<Result<Self::Transform, Self::InitError>>;\n"
    )
    .unwrap();
    writeln!(
        transform_impl,
        "{INDENT}fn new_transform(&self, service: S) -> Self::Future {{"
    )
    .unwrap();
    writeln!(
        transform_impl,
        "{INDENT}{INDENT}ready(Ok({service_name}Middleware {{"
    )
    .unwrap();
    writeln!(
        transform_impl,
        "{INDENT}{INDENT}{INDENT}service: Rc::new(service),"
    )
    .unwrap();
    writeln!(
        transform_impl,
        "{INDENT}{INDENT}{INDENT}guard: self.guard.clone(),"
    )
    .unwrap();
    writeln!(transform_impl, "{INDENT}{INDENT}}}))").unwrap();
    writeln!(transform_impl, "{INDENT}}}\n}}\n").unwrap();

    /*
     * Middleware struct statement
     */
    let mut mw_strct_stmt = format!("pub(crate) struct {service_name}Middleware");
    write_letter_bounds(&mut mw_strct_stmt, apis, &["S"]);
    if !apis.is_empty() {
        writeln!(mw_strct_stmt, "\nwhere").unwrap();
        for c in apis {
            writeln!(
                mw_strct_stmt,
                "{INDENT}{}: {}Api,",
                &uppercase(c)[..1],
                uppercase(c)
            )
            .unwrap();
        }
    }
    if !apis.is_empty() {
        write!(mw_strct_stmt, "{{\n{INDENT}guard: Rc<{service_name}").unwrap();
    } else {
        write!(mw_strct_stmt, " {{\n{INDENT}guard: Rc<{service_name}").unwrap();
    }
    if !apis.is_empty() {
        write_letter_bounds(&mut mw_strct_stmt, apis, &[]);
    }
    writeln!(mw_strct_stmt, ">,\n{INDENT}service: Rc<S>,\n}}\n").unwrap();

    /*
     * Impl Service statement
     */
    let mut service_impl = String::from("impl");
    write_letter_bounds(&mut service_impl, apis, &["S"]);
    write!(
        service_impl,
        " Service<ServiceRequest> for {service_name}Middleware"
    )
    .unwrap();
    write_letter_bounds(&mut service_impl, apis, &["S"]);
    writeln!(service_impl, "\nwhere").unwrap();
    writeln!(service_impl,"{INDENT}S: Service<ServiceRequest, Response = ServiceResponse, Error = actix_web::Error> + 'static,").unwrap();
    writeln!(service_impl, "{INDENT}S::Future: 'static,").unwrap();
    if !apis.is_empty() {
        for c in apis {
            writeln!(
                service_impl,
                "{INDENT}{}: {}Api + Send + Sync + 'static,",
                &uppercase(c)[..1],
                uppercase(c)
            )
            .unwrap();
        }
    }
    writeln!(service_impl, "{{").unwrap();
    writeln!(service_impl, "{INDENT}type Response = ServiceResponse;").unwrap();
    writeln!(service_impl, "{INDENT}type Error = actix_web::Error;").unwrap();
    writeln!(
        service_impl,
        "{INDENT}type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;\n"
    )
    .unwrap();
    writeln!(service_impl, "{INDENT}forward_ready!(service);\n").unwrap();
    writeln!(
        service_impl,
        "{INDENT}fn call(&self, req: ServiceRequest) -> Self::Future {{"
    )
    .unwrap();
    writeln!(
        service_impl,
        "{INDENT}{INDENT}let guard = self.guard.clone();"
    )
    .unwrap();
    writeln!(
        service_impl,
        "{INDENT}{INDENT}let service = self.service.clone();\n"
    )
    .unwrap();
    writeln!(service_impl, "{INDENT}{INDENT}async move {{").unwrap();
    writeln!(service_impl, "{INDENT}{INDENT}{INDENT}let res = service.call(req).await?;\n{INDENT}{INDENT}{INDENT}Ok(res)").unwrap();
    writeln!(
        service_impl,
        "{INDENT}{INDENT}}}\n{INDENT}{INDENT}.boxed_local()\n{INDENT}}}\n}}"
    )
    .unwrap();

    write!(
        buf,
        "{use_stmt}{struct_stmt}{transform_impl}{mw_strct_stmt}{service_impl}"
    )
    .unwrap();
}
