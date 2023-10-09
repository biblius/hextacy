use crate::core::repository::user::UserRepository;
use crate::error::Error;
use async_trait::async_trait;
use hextacy::queue::QueueHandler;
use hextacy::{component, Driver};
use serde::{Deserialize, Serialize};
use tracing::info;

// A generic implementation for a handler. This should be passed in to the consumer's runtime
// upon calling its `start` method.
#[component(
    use Driver as driver,
    use Users
)]
pub struct HelloWorldHandler {}

/// The implementation that gets called in the consumer's runtime and handles the message.
#[async_trait]
impl<D, U> QueueHandler<HelloWorld> for HelloWorldHandler<D, U>
where
    D: Driver + Send + Sync,
    D::Connection: Send,
    U: UserRepository<D::Connection> + Send + Sync,
{
    type Error = Error;
    async fn handle(&mut self, message: HelloWorld) -> Result<(), Self::Error> {
        let mut conn = self.driver.connect().await?;

        match message {
            HelloWorld::Hello(GreetUser { id, greeting }) => {
                let Some(user) = self.users.get_by_id(&mut conn, id).await? else {
                    // Normally we would handle this scenario by notifying whoever sent the greet or something
                    return Ok(());
                };
                info!("{greeting}, {}", user.username);
            }
            HelloWorld::World(RegisterUser {
                ref username,
                ref password,
            }) => {
                let user = self.users.create(&mut conn, username, password).await?;
                info!("Successfully created user {user:?}");
            }
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum HelloWorld {
    Hello(GreetUser),
    World(RegisterUser),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterUser {
    pub username: String,
    /// For example only, don't do this at home
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GreetUser {
    id: hextacy::exports::uuid::Uuid,
    greeting: String,
}

/* let publisher = amqp.publisher_default("my-queue").await.unwrap();
         let publisher = HelloWorldPublisher {
    queue: "my-queue".to_string(),
    exchange: None,
    channel: publisher,
};

let consumer = amqp.consumer_default("my-queue", "me").await.unwrap();
let handler = HelloWorldHandler {
    driver: repository.clone(),
    users: UserAdapter,
};

let stop_handle = consumer.start(handler);

publisher
    .publish(HelloWorld::World(RegisterUser {
        username: "lmao".to_string(),
        password: "supersecret".to_string(),
    }))
    .await
    .unwrap(); */
