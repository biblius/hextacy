// use reqwest::{header, StatusCode};
use std::fs;
use tracing::debug;

lazy_static::lazy_static! {
    pub static ref FAVICON: Vec<u8> = {
        debug!("Loading favicon");
        fs::read("resources/favicon.ico").expect("Couldn't load favicon.ico")
    };
}

pub async fn favicon() -> Vec<u8> {
    FAVICON.clone()
}
