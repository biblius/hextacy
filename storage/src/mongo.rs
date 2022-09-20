use mongodb::{
    options::{ClientOptions, Credential, ServerAddress},
    sync::Client as SyncClient,
    Client,
};
use tracing::trace;

pub fn client_options() -> ClientOptions {
    let mut params = config::get_multiple(vec![
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

/// Creates a `ClientOptions` struct with the `MONGO_HOST` and `MONGO_PORT` environment variables
/// as the address. Panics if they are not set
pub fn default_client_options() -> ClientOptions {
    let mut params = config::get_multiple(vec!["MONGO_HOST", "MONGO_PORT"]);

    let port = params.pop().map_or_else(
        || 27017,
        |p| {
            p.parse::<u16>()
                .expect("MONGO_PORT must be a valid integer")
        },
    );

    let host = params.pop().expect("MONGO_HOST must be set");

    let address = ServerAddress::Tcp {
        host,
        port: Some(port),
    };

    ClientOptions::builder().hosts(vec![address]).build()
}

pub struct Mongo {
    pub client: Client,
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
    pub fn list_dbs(&self) {
        for db in self.client.list_database_names(None, None).unwrap() {
            println!("{}", db)
        }
    }
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

    pub async fn list_dbs(&self) {
        self.client.list_database_names(None, None).await.unwrap();
    }
}

impl Default for Mongo {
    fn default() -> Self {
        Self::new()
    }
}
