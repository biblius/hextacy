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

pub fn contracts(buf: &mut String, contracts: &[&str], ty: BoilerType) {
    let vis = match ty {
        BoilerType::Route => "super",
        BoilerType::MW => "crate",
    };
    writeln!(
        buf,
        "#[cfg_attr(test, mockall::automock)]\npub({vis}) trait ServiceContract {{}}"
    )
    .unwrap();
    for contract in contracts {
        writeln!(
            buf,
            "\n#[cfg_attr(test, mockall::automock)]\npub({vis}) trait {}Contract {{}}",
            uppercase(contract)
        )
        .unwrap();
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
            "pub(super) struct {} {{}}\n\nimpl {}Contract for {} {{}}",
            uppercase(contracts[0]),
            uppercase(contracts[0]),
            uppercase(contracts[0]),
        )
        .unwrap();
    } else {
        write!(use_stmt, "{{").unwrap();
        for (i, contract) in contracts.iter().enumerate() {
            if i == contracts.len() - 1 {
                writeln!(use_stmt, "{}Contract}};", uppercase(contract)).unwrap();
            } else {
                write!(use_stmt, "{}Contract, ", uppercase(contract)).unwrap();
            }
            writeln!(
                struct_stmt,
                "pub(super) struct {} {{}}\n\nimpl {}Contract for {} {{}}\n",
                uppercase(contract),
                uppercase(contract),
                uppercase(contract),
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

pub(crate) fn write_letter_bounds(stmt: &mut String, contracts: &[&str], append: &[&str]) {
    write!(stmt, "<").unwrap();
    for (i, a) in append.iter().enumerate() {
        if contracts.is_empty() {
            if i == append.len() - 1 {
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
pub(crate) fn write_contracts_use(buf: &mut String, contracts: &[&str]) {
    for (i, c) in contracts.iter().enumerate() {
        if i == contracts.len() - 1 {
            write!(buf, "{}Contract", uppercase(c)).unwrap();
        } else {
            write!(buf, "{}Contract, ", uppercase(c)).unwrap();
        }
    }
    writeln!(buf, "}};").unwrap();
}

pub fn domain(buf: &mut String, service_name: &str, contracts: &[&str]) {
    // Use statement
    let mut use_stmt = String::from("use super::contract::");
    if !contracts.is_empty() {
        write!(use_stmt, "{{ServiceContract, ").unwrap();
        write_contracts_use(&mut use_stmt, contracts);
    } else {
        writeln!(use_stmt, "ServiceContract {{}}").unwrap();
    }

    // Struct statement
    let mut struct_stmt = format!("#[derive(Debug)]\npub(super) struct {service_name}");
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
        writeln!(struct_stmt, "{{").unwrap();
        for contract in contracts {
            writeln!(
                struct_stmt,
                "{INDENT}pub {contract}: {},",
                &uppercase(contract)[..1],
            )
            .unwrap();
        }
        writeln!(struct_stmt, "}}\n").unwrap();
    } else {
        writeln!(struct_stmt, ";\n").unwrap();
    }

    // Impl statement
    let mut impl_stmt = String::from("impl");
    if !contracts.is_empty() {
        write_letter_bounds(&mut impl_stmt, contracts, &[]);
    }
    write!(impl_stmt, " ServiceContract for {service_name}").unwrap();
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

    writeln!(buf, "{use_stmt}{struct_stmt}{impl_stmt}",).unwrap();
}
