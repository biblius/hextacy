use super::INDENT;
use crate::uppercase;
use std::fmt::Write;

pub fn setup(buf: &mut String, service_name: &str, contracts: &[&str]) {
    // Use statement
    let mut use_stmt =
        format!("use super::{{\n{INDENT}domain::{service_name},\n{INDENT}handler,\n");
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

    write!(buf, "{use_stmt}{fn_stmt}").unwrap();
}
