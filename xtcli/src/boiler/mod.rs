use crate::{uppercase, MW_FILES, ROUTE_FILES};
use std::fmt::Write;

pub mod files;
pub mod middleware;
pub mod router;

pub(crate) const INDENT: &str = "   ";

pub enum BoilerType {
    Route,
    MW,
}

pub fn apis(buf: &mut String, apis: &[&str], ty: BoilerType) {
    let vis = match ty {
        BoilerType::Route => "super",
        BoilerType::MW => "crate",
    };
    writeln!(
        buf,
        "#[cfg_attr(test, mockall::automock)]\npub({vis}) trait ServiceApi {{}}"
    )
    .unwrap();
    for api in apis {
        writeln!(
            buf,
            "\n#[cfg_attr(test, mockall::automock)]\npub({vis}) trait {}Api {{}}",
            uppercase(api)
        )
        .unwrap();
    }
}

pub fn infrastructure(buf: &mut String, apis: &[&str]) {
    let mut use_stmt = String::new();
    let mut struct_stmt = String::new();
    write!(use_stmt, "use super::api::").unwrap();
    if apis.len() == 1 {
        writeln!(use_stmt, "{}Api;", uppercase(apis[0])).unwrap();
        writeln!(
            struct_stmt,
            "pub(super) struct {} {{}}\n\nimpl {}Api for {} {{}}",
            uppercase(apis[0]),
            uppercase(apis[0]),
            uppercase(apis[0]),
        )
        .unwrap();
    } else {
        write!(use_stmt, "{{").unwrap();
        for (i, api) in apis.iter().enumerate() {
            if i == apis.len() - 1 {
                writeln!(use_stmt, "{}Api}};", uppercase(api)).unwrap();
            } else {
                write!(use_stmt, "{}Api, ", uppercase(api)).unwrap();
            }
            writeln!(
                struct_stmt,
                "pub(super) struct {} {{}}\n\nimpl {}Api for {} {{}}\n",
                uppercase(api),
                uppercase(api),
                uppercase(api),
            )
            .unwrap();
        }
    }
    write!(buf, "{use_stmt}{struct_stmt}").unwrap();
}

pub fn r#mod(buf: &mut String, ty: BoilerType) {
    match ty {
        BoilerType::Route => {
            for file in ROUTE_FILES {
                if file != "mod" {
                    if file == "setup" {
                        writeln!(buf, "pub(crate) mod {file};").unwrap();
                    } else {
                        writeln!(buf, "pub(super) mod {file};").unwrap();
                    }
                }
            }
            write!(buf, "\n#[cfg(test)]\nmod tests {{\n\n{INDENT}#[test]\n{INDENT}fn test() {{\n{INDENT}{INDENT}assert_eq!(2 + 2, 4)\n{INDENT}}}\n}}").unwrap();
        }
        BoilerType::MW => {
            for file in MW_FILES {
                if file != "mod" {
                    if file == "interceptor" {
                        writeln!(buf, "pub(crate) mod {file};").unwrap();
                    } else {
                        writeln!(buf, "pub(super) mod {file};").unwrap();
                    }
                }
            }
            write!(buf, "\n#[cfg(test)]\nmod tests {{\n\n{INDENT}#[test]\n{INDENT}fn test() {{\n{INDENT}{INDENT}assert_eq!(2 + 2, 4)\n{INDENT}}}\n}}").unwrap();
        }
    }
}

pub(crate) fn write_letter_bounds(stmt: &mut String, apis: &[&str], append: &[&str]) {
    write!(stmt, "<").unwrap();
    for (i, a) in append.iter().enumerate() {
        if apis.is_empty() {
            if i == append.len() - 1 {
                write!(stmt, "{a}").unwrap();
            } else {
                write!(stmt, "{a}, ").unwrap();
            }
        } else {
            write!(stmt, "{a}, ").unwrap();
        }
    }
    for (i, c) in apis.iter().enumerate() {
        if i == apis.len() - 1 {
            write!(stmt, "{}", &uppercase(c)[..1]).unwrap();
        } else {
            write!(stmt, "{}, ", &uppercase(c)[..1]).unwrap();
        }
    }
    write!(stmt, ">").unwrap();
}

/// Writes the closing `};` tokens as well
pub(crate) fn write_apis_use(buf: &mut String, apis: &[&str]) {
    for (i, c) in apis.iter().enumerate() {
        if i == apis.len() - 1 {
            write!(buf, "{}Api", uppercase(c)).unwrap();
        } else {
            write!(buf, "{}Api, ", uppercase(c)).unwrap();
        }
    }
    writeln!(buf, "}};").unwrap();
}

pub fn domain(buf: &mut String, service_name: &str, apis: &[&str]) {
    // Use statement
    let mut use_stmt = String::from("use super::api::");
    if !apis.is_empty() {
        write!(use_stmt, "{{ServiceApi, ").unwrap();
        write_apis_use(&mut use_stmt, apis);
    } else {
        writeln!(use_stmt, "ServiceApi {{}}").unwrap();
    }

    // Struct statement
    let mut struct_stmt = format!("#[derive(Debug)]\npub(super) struct {service_name}");
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
        writeln!(struct_stmt, "{{").unwrap();
        for api in apis {
            writeln!(struct_stmt, "{INDENT}pub {api}: {},", &uppercase(api)[..1],).unwrap();
        }
        writeln!(struct_stmt, "}}\n").unwrap();
    } else {
        writeln!(struct_stmt, ";\n").unwrap();
    }

    // Impl statement
    let mut impl_stmt = String::from("impl");
    if !apis.is_empty() {
        write_letter_bounds(&mut impl_stmt, apis, &[]);
    }
    write!(impl_stmt, " ServiceApi for {service_name}").unwrap();
    if !apis.is_empty() {
        write_letter_bounds(&mut impl_stmt, apis, &[]);
        writeln!(impl_stmt, "\nwhere").unwrap();
        for c in apis {
            writeln!(
                impl_stmt,
                "{INDENT}{}: {}Api + Send + Sync,",
                &uppercase(c)[..1],
                uppercase(c)
            )
            .unwrap();
        }
        write!(impl_stmt, "{{\n}}").unwrap();
    } else {
        write!(impl_stmt, " {{}}").unwrap();
    }

    writeln!(buf, "{use_stmt}{struct_stmt}{impl_stmt}",).unwrap();
}
