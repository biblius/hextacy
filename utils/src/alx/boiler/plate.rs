use crate::{uppercase, FILES, INDENT};
use std::fmt::Write;

pub fn r#mod(buf: &mut String) {
    for f in FILES {
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

pub fn contracts(buf: &mut String, contracts: &[&str]) {
    writeln!(buf, "use async_trait::async_trait;\n").unwrap();
    writeln!(buf, "#[cfg_attr(test, mockall::automock)]\n#[async_trait]\npub(super) trait ServiceContract {{}}").unwrap();
    for c in contracts {
        writeln!(
            buf,
            "\n#[cfg_attr(test, mockall::automock)]\n#[async_trait]\npub(super) trait {}Contract {{}}",
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
    // Utility closure
    let write_bounds = |stmt: &mut String| {
        write!(stmt, "<").unwrap();
        for (i, c) in contracts.iter().enumerate() {
            if i == contracts.len() - 1 {
                write!(stmt, "{}", &uppercase(c)[..1]).unwrap();
            } else {
                write!(stmt, "{}, ", &uppercase(c)[..1]).unwrap();
            }
        }
        write!(stmt, "> ").unwrap();
    };

    // Use statement
    let mut use_stmt = String::from("use super::contract::");
    if !contracts.is_empty() {
        write!(use_stmt, "{{ServiceContract, ").unwrap();
        for (i, c) in contracts.iter().enumerate() {
            if i == contracts.len() - 1 {
                write!(use_stmt, "{}Contract", uppercase(c)).unwrap();
            } else {
                write!(use_stmt, "{}Contract, ", uppercase(c)).unwrap();
            }
        }
        writeln!(use_stmt, "}};").unwrap();
    } else {
        writeln!(use_stmt, "ServiceContract;").unwrap();
    }
    writeln!(use_stmt, "use async_trait::async_trait;\n").unwrap();

    // Struct statement
    let mut struct_statement = format!("pub(super) struct {}", service_name);
    if !contracts.is_empty() {
        write_bounds(&mut struct_statement);
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
        write_bounds(&mut impl_stmt);
    }
    write!(impl_stmt, "ServiceContract for {}", service_name).unwrap();
    if !contracts.is_empty() {
        write_bounds(&mut impl_stmt);
        write!(impl_stmt, "\nwhere\n").unwrap();
        for c in contracts {
            writeln!(
                impl_stmt,
                "{INDENT}{}: {}Contract + Send + Sync,",
                &uppercase(c)[..1],
                uppercase(c)
            )
            .unwrap();
        }
    }
    write!(impl_stmt, "{{\n}}").unwrap();

    writeln!(buf, "{}{}{}", use_stmt, struct_statement, impl_stmt).unwrap();
}
