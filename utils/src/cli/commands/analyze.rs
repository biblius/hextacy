use crate::{
    config::{Endpoint, ProjectConfig, Resource, Route, Scope},
    error::AlxError,
};
use std::{
    fs::{self, DirEntry},
    path::Path,
};

/// Recursively read the file system at the specified path.
pub fn router_read_recursive(
    pc: &mut ProjectConfig,
    dir: &Path,
    cb: &dyn Fn(&DirEntry, &mut ProjectConfig) -> Result<(), AlxError>,
) -> Result<(), AlxError> {
    println!(
        "\u{1F963} Reading {} \u{1F963}",
        dir.to_str().expect("Couldn't read directory name")
    );
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        println!("Found entry: {:?}", entry);
        let path = entry.path();
        if path.is_dir() {
            router_read_recursive(pc, &path, cb)?;
        } else {
            // println!("Found {:?}.{:?}", entry.file_name(), entry.file_type());
            // println!(
            //     "Name as String: {:?}",
            //     entry.file_name().into_string().unwrap()
            // );
            if entry
                .file_name()
                .into_string()
                .unwrap()
                .contains("setup.rs")
            {
                cb(&entry, pc).unwrap();
            }
        }
    }
    Ok(())
}

pub fn populate_config(entry: &DirEntry, pc: &mut ProjectConfig) -> Result<(), AlxError> {
    let file = fs::read_to_string(Path::new(&entry.path()))?;
    let mut in_scope = false;
    let mut scope_buf = Scope::default();
    for l in file.lines() {
        if l.contains("web::scope") {
            in_scope = true;
            scope_buf.name = l
                [l.find('(').expect("Invalid scope") + 2..l.find(')').expect("Invalid scope") - 1]
                .to_string();
            println!("Found scope: {}", scope_buf.name);
        }
        if l.contains(");") && in_scope {
            in_scope = false;
            pc.endpoints.push(Endpoint::Scope(scope_buf));
            scope_buf = Scope::default();
        }
        if l.contains("web::resource") {
            // Extract behind
            let path = l
                .trim()
                .split("web::resource")
                .nth(1)
                .expect("Invalid resource")
                .to_string();
            // Grab only the inner stuff in the brackets without the quotes
            let path = path[path.find('(').expect("Invalid resource") + 2
                ..path.find(')').expect("Invalid resource") - 1]
                .to_string();
            if in_scope {
                scope_buf.resources.push(Resource {
                    path,
                    routes: vec![Route {
                        method: l.to_string(),
                        handler: "yolo.rs".to_string(),
                        extractors: None,
                        middleware: None,
                    }],
                    middleware: None,
                });
            } else {
                pc.endpoints.push(Endpoint::Resource(Resource {
                    path,
                    routes: vec![Route {
                        method: l.to_string(),
                        handler: "yolo.rs".to_string(),
                        extractors: None,
                        middleware: None,
                    }],
                    middleware: None,
                }));
            }
        }
    }
    Ok(())
}
