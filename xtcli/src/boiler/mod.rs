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

pub fn components(buf: &mut String, components: &[&str], ty: BoilerType) {
    let vis = match ty {
        BoilerType::Route => "super",
        BoilerType::MW => "crate",
    };
    writeln!(
        buf,
        "#[cfg_attr(test, mockall::automock)]\npub({vis}) trait ServiceApi {{}}"
    )
    .unwrap();
    for component in components {
        writeln!(
            buf,
            "\n#[cfg_attr(test, mockall::automock)]\npub({vis}) trait {}Api {{}}",
            uppercase(component)
        )
        .unwrap();
    }
}

pub fn infrastructure(buf: &mut String, components: &[&str]) {
    let mut use_stmt = String::new();
    let mut struct_stmt = String::new();
    write!(use_stmt, "use super::api::").unwrap();
    if components.len() == 1 {
        writeln!(use_stmt, "{}Api;", uppercase(components[0])).unwrap();
        writeln!(
            struct_stmt,
            "pub(super) struct {} {{}}\n\nimpl {}Api for {} {{}}",
            uppercase(components[0]),
            uppercase(components[0]),
            uppercase(components[0]),
        )
        .unwrap();
    } else {
        write!(use_stmt, "{{").unwrap();
        for (i, api) in components.iter().enumerate() {
            if i == components.len() - 1 {
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

pub(crate) fn write_letter_bounds(stmt: &mut String, components: &[&str], append: &[&str]) {
    write!(stmt, "<").unwrap();
    for (i, a) in append.iter().enumerate() {
        if components.is_empty() {
            if i == append.len() - 1 {
                write!(stmt, "{a}").unwrap();
            } else {
                write!(stmt, "{a}, ").unwrap();
            }
        } else {
            write!(stmt, "{a}, ").unwrap();
        }
    }
    for (i, c) in components.iter().enumerate() {
        if i == components.len() - 1 {
            write!(stmt, "{}", &uppercase(c)[..1]).unwrap();
        } else {
            write!(stmt, "{}, ", &uppercase(c)[..1]).unwrap();
        }
    }
    write!(stmt, ">").unwrap();
}

/// Writes the closing `};` tokens as well
pub(crate) fn write_apis_use(buf: &mut String, components: &[&str]) {
    for (i, c) in components.iter().enumerate() {
        if i == components.len() - 1 {
            write!(buf, "{}Api", uppercase(c)).unwrap();
        } else {
            write!(buf, "{}Api, ", uppercase(c)).unwrap();
        }
    }
    writeln!(buf, "}};").unwrap();
}

pub fn domain(buf: &mut String, service_name: &str, components: &[&str]) {
    // Use statement
    let mut use_stmt = String::from("use super::api::");
    if !components.is_empty() {
        write!(use_stmt, "{{ServiceApi, ").unwrap();
        write_apis_use(&mut use_stmt, components);
    } else {
        writeln!(use_stmt, "ServiceApi {{}}").unwrap();
    }

    // Struct statement
    let mut struct_stmt = format!("#[derive(Debug)]\npub(super) struct {service_name}");
    if !components.is_empty() {
        write_letter_bounds(&mut struct_stmt, components, &[]);
        writeln!(struct_stmt, "\nwhere").unwrap();
        for c in components {
            writeln!(
                struct_stmt,
                "{INDENT}{}: {}Api,",
                &uppercase(c)[..1],
                uppercase(c)
            )
            .unwrap();
        }
        writeln!(struct_stmt, "{{").unwrap();
        for api in components {
            writeln!(struct_stmt, "{INDENT}pub {api}: {},", &uppercase(api)[..1],).unwrap();
        }
        writeln!(struct_stmt, "}}\n").unwrap();
    } else {
        writeln!(struct_stmt, ";\n").unwrap();
    }

    // Impl statement
    let mut impl_stmt = String::from("impl");
    if !components.is_empty() {
        write_letter_bounds(&mut impl_stmt, components, &[]);
    }
    write!(impl_stmt, " ServiceApi for {service_name}").unwrap();
    if !components.is_empty() {
        write_letter_bounds(&mut impl_stmt, components, &[]);
        writeln!(impl_stmt, "\nwhere").unwrap();
        for c in components {
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
