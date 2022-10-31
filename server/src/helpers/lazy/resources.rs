use lazy_static::lazy_static;
use std::{fs, path::Path};
use tracing::debug;

lazy_static! {
    pub static ref FAVICON: Vec<u8> = {
        debug!("Loading favicon");
        fs::read(Path::new("resources/favicon.ico")).expect("Couldn't load favicon.ico")
    };
}
