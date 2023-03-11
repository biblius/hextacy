use super::DBConnect;
use crate::clients::ClientError;
use async_trait::async_trait;
use mongodb::{
    options::{ClientOptions, Credential, ServerAddress},
    Client, ClientSession,
};
use tracing::trace;

pub struct Mongo {
    pub client: Client,
}

impl Mongo {
    pub fn new(host: &str, port: u16, user: &str, password: &str, db: &str) -> Self {
        let options = client_options(host, port, user, password, db);
        match Client::with_options(options) {
            Ok(client) => {
                trace!("Built Mongo client");
                Self { client }
            }
            Err(e) => panic!("Error occurred while building Mongo client: {e}"),
        }
    }

    pub fn direct(host: &str, port: u16, user: &str, password: &str, db: &str) -> Self {
        let mut options = client_options(host, port, user, password, db);
        options.direct_connection = true.into();
        match Client::with_options(options) {
            Ok(client) => {
                trace!("Built Mongo client with direct connection");
                Self { client }
            }
            Err(e) => panic!("Error occurred while building sync Mongo client: {e}"),
        }
    }
}

fn client_options(host: &str, port: u16, user: &str, password: &str, db: &str) -> ClientOptions {
    let address = ServerAddress::Tcp {
        host: host.to_string(),
        port: Some(port),
    };

    let credential = Credential::builder()
        .password(password.to_string())
        .username(user.to_string())
        .build();

    trace!("Building Mongo client options with {address}");

    ClientOptions::builder()
        .hosts(vec![address])
        .credential(credential)
        .default_database(db.to_string())
        .build()
}

#[async_trait(?Send)]
impl DBConnect for Mongo {
    type Connection = ClientSession;

    async fn connect(&self) -> Result<Self::Connection, ClientError> {
        trace!("Mongo - Attempting pooled connection");
        let session = self.client.start_session(None).await?;
        Ok(session)
    }
}
