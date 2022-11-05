use colored::Colorize;
use std::{
    fs,
    io::{stdin, Write},
};

use crate::print;

/// Returns false if the user aborted the process
pub fn handle_create_dir(path: &str) -> bool {
    match fs::create_dir(path) {
        Ok(_) => {}
        Err(e) => {
            println!(
                "An error occurred: {:?}\n\
                Would you like to continue anyway?\
                \n\u{26A0} {} \u{26A0}\n\
                This will completely overwrite {}",
                e.kind(),
                "WARNING".red(),
                path
            );
            let mut buf = String::new();
            loop {
                println!("Press [y]es or [n]o to continue or abort");
                stdin().read_line(&mut buf).expect("Couldn't parse input");
                match buf.trim() {
                    "y" | "yes" => {
                        println!("Overwriting {}", path);
                        fs::remove_dir_all(path).expect("Couldn't remove directory");
                        fs::create_dir(path).expect("Couldn't create directory");
                        break;
                    }
                    "n" | "no" => {
                        println!("Aborting");
                        return false;
                    }
                    _ => {
                        buf.clear();
                    }
                }
            }
        }
    };
    true
}

pub fn write_to_mod_file(file_path: &str, mod_name: &str) {
    print(&format!(
        "{} Adding {} to {}",
        "\u{270E}".green(),
        mod_name,
        file_path
    ));
    let f = fs::read_to_string(file_path).unwrap();
    let mut new_file_contents = format!("pub(crate) mod {};\n", mod_name);
    if !f.contains(&new_file_contents) {
        new_file_contents.push_str(&f);
        fs::remove_file(file_path).expect("Couldn't remove old mod.rs file");

        let mut new_file = fs::File::create(file_path).expect("Couldn't create file");
        new_file.write_all(new_file_contents.as_bytes()).unwrap();
    }
}
