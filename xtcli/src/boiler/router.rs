use super::INDENT;
use crate::uppercase;
use std::fmt::Write;

pub fn setup(buf: &mut String, service_name: &str, apis: &[&str]) {
    // Use statement
    let mut use_stmt =
        format!("use super::{{\n{INDENT}domain::{service_name},\n{INDENT}handler,\n");
    if !apis.is_empty() {
        if apis.len() == 1 {
            writeln!(
                use_stmt,
                "{INDENT}infrastructure::{},\n}};",
                uppercase(apis[0])
            )
            .unwrap();
        } else {
            write!(use_stmt, "{INDENT}infrastructure::{{").unwrap();
            for (i, c) in apis.iter().enumerate() {
                if i == apis.len() - 1 {
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
