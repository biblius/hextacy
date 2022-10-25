use crate::config::env;
use mongodb::{
    options::{ClientOptions, Credential, ServerAddress},
    sync::Client as SyncClient,
    Client,
};
use tracing::trace;

/// Searches for `MONGO_HOST`, `MONGO_PORT`, `MONGO_USER`, `MONGO_PASSWORD`,
/// and `MONDO_DATABASE` environment variables and panics if any are not set.
pub fn client_options() -> ClientOptions {
    let mut params = env::get_multiple(&[
        "MONGO_HOST",
        "MONGO_PORT",
        "MONGO_USER",
        "MONGO_PASSWORD",
        "MONGO_DATABASE",
    ]);

    let database = params.pop().expect("MONGO_DATABASE must be set");

    let password = params.pop().expect("MONGO_PASSWORD must be set");

    let user = params.pop().expect("MONGO_USER must be set");

    let port = params
        .pop()
        .expect("MONGO_PORT must be set")
        .parse()
        .expect("MONGO_PORT must be a valid integer");

    let host = params.pop().expect("MONGO_HOST must be set");

    let address = ServerAddress::Tcp {
        host,
        port: Some(port),
    };

    let credential = Credential::builder()
        .password(password)
        .username(user)
        .build();

    trace!("Building Mongo client options with {}", address);

    ClientOptions::builder()
        .hosts(vec![address])
        .credential(credential)
        .default_database(database)
        .build()
}

pub struct Mongo {
    pub client: Client,
}

impl Mongo {
    pub fn new() -> Self {
        match Client::with_options(client_options()) {
            Ok(client) => {
                trace!("Built Mongo client");
                Self { client }
            }
            Err(e) => panic!("Error occurred while building Mongo client: {e}"),
        }
    }

    pub fn direct() -> Self {
        let mut opts = client_options();
        opts.direct_connection = Some(true);
        match Client::with_options(opts) {
            Ok(client) => {
                trace!("Built Mongo client with direct connection");
                Self { client }
            }
            Err(e) => panic!("Error occurred while building sync Mongo client: {e}"),
        }
    }
}

impl Default for Mongo {
    fn default() -> Self {
        Self::new()
    }
}

pub struct MongoSync {
    pub client: SyncClient,
}

impl MongoSync {
    pub fn new() -> Self {
        match SyncClient::with_options(client_options()) {
            Ok(client) => {
                trace!("Built sync Mongo client");
                Self { client }
            }
            Err(e) => panic!("Error occurred while building sync Mongo client: {e}"),
        }
    }
    pub fn direct() -> Self {
        let mut opts = client_options();
        opts.direct_connection = Some(true);
        match SyncClient::with_options(opts) {
            Ok(client) => {
                trace!("Built sync Mongo client with direct connection");
                Self { client }
            }
            Err(e) => panic!("Error occurred while building sync Mongo client: {e}"),
        }
    }
}

impl Default for MongoSync {
    fn default() -> Self {
        Self::new()
    }
}
