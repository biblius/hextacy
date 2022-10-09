use reqwest::Client;

pub fn init() -> Client {
    let client_builder = reqwest::ClientBuilder::new();
    client_builder.build().expect("Failed to build client")
}
