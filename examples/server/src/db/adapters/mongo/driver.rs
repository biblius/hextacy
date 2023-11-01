use mongodb::{
    options::{ClientOptions, Credential, ServerAddress},
    Client,
};
use tracing::trace;

/// Thin wrapper around a [mongodb::Client] that implements [Driver] that can be
/// injected into services.
#[derive(Debug, Clone)]
pub struct MongoDriver {
    pub driver: Client,
}

impl MongoDriver {
    pub fn new(host: &str, port: u16, user: &str, password: &str, db: &str) -> Self {
        let address = ServerAddress::Tcp {
            host: host.to_string(),
            port: Some(port),
        };

        let credential = Credential::builder()
            .password(password.to_string())
            .username(user.to_string())
            .build();

        let options = ClientOptions::builder()
            .hosts(vec![address])
            .credential(credential)
            .default_database(db.to_string())
            .build();

        let client = match Client::with_options(options) {
            Ok(driver) => {
                trace!("Built Mongo driver");
                Self { driver }
            }
            Err(e) => panic!("Error occurred while building Mongo driver: {e}"),
        };

        tracing::debug!(
            "Successfully initialised Mongo client at {}",
            format!("mongodb://{user}:***@{host}:{port}/{db}")
        );

        client
    }
}
