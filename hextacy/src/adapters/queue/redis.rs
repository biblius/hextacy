use async_trait::async_trait;
use deadpool_redis::redis::{
    aio::Connection, AsyncCommands, Client, IntoConnectionInfo, Msg, RedisError,
};
use futures_util::{Stream, StreamExt};
use serde::{de::DeserializeOwned, Serialize};
use std::{fmt::Debug, pin::Pin};

use crate::queue::{Consumer, Producer, QueueError};

/// A wrapper around a [redis client][deadpool_redis::redis::Client] with simple functionality
/// for creating queue publishers and consumers.
///
/// ### Example
///
/// ```ignore
/// struct MyMessageHandler {}
///
/// enum MyMessage {
///     SomeVariant
/// }
///
/// enum MyError {}
///
/// #[async_trait]
/// impl QueueHandler<MyMessage> for MyMessageHandler {
///   type Error = MyError;
///   async fn handle(&mut self, message: HelloWorld) -> Result<(), Self::Error> {
///     // ...
///   }
/// }
///
/// let redis_q = RedisMessageQueue::new(/* ... */);
/// let mut publisher = redis_q.publisher("my-queue").await.unwrap();
/// let consumer = redis_q.consumer("my-queue").await.unwrap();
/// consumer.start(MyMessage);
///
/// publisher.publish(MyMessage::SomeVariant);
/// ```
#[derive(Debug, Clone)]
pub struct RedisMessageQueue {
    client: Client,
}

impl RedisMessageQueue {
    pub fn new(host: &str, port: u16, user: Option<&str>, password: Option<&str>) -> Self {
        let db_url = format!("redis://{host}:{port}");
        let mut conn_info = db_url.clone().into_connection_info().unwrap();
        conn_info.redis.password = password.map(|pw| pw.to_string());
        conn_info.redis.username = user.map(|uname| uname.to_string());
        let client = Client::open(conn_info).expect("Could not create Redis client");
        Self { client }
    }

    pub async fn publisher(&self, channel: &str) -> Result<RedisPublisher, RedisError> {
        let conn = self.client.get_async_connection().await?;
        Ok(RedisPublisher {
            channel: channel.to_string(),
            connection: conn,
        })
    }

    pub async fn consumer(&self, channel: &str) -> Result<RedisConsumer, RedisError> {
        let conn = self.client.get_async_connection().await?;
        let mut pubsub = conn.into_pubsub();
        pubsub.subscribe(channel).await?;
        Ok(RedisConsumer {
            stream: Box::pin(pubsub.into_on_message()),
        })
    }
}

pub struct RedisPublisher {
    channel: String,
    connection: Connection,
}

impl Debug for RedisPublisher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisPublisher")
            .field("channel", &self.channel)
            .field("connection", &"{ ... }")
            .finish()
    }
}

#[async_trait]
impl Producer for RedisPublisher {
    type Error = QueueError<RedisError>;
    async fn publish<M>(&mut self, message: M) -> Result<(), Self::Error>
    where
        M: Serialize + Send + Sync + 'static,
    {
        let message = serde_json::to_string(&message)?;
        self.connection
            .publish(self.channel.as_str(), message)
            .await
            .map_err(QueueError::Driver)
    }
}

pub struct RedisConsumer {
    stream: Pin<Box<dyn Stream<Item = Msg> + Send>>,
}

#[async_trait]
impl<M> Consumer<M> for RedisConsumer
where
    M: DeserializeOwned + Send + 'static,
{
    type Error = QueueError<RedisError>;

    async fn poll_queue(&mut self) -> Result<Option<M>, Self::Error> {
        let Some(message) = self.stream.next().await else {
            return Ok(None);
        };
        let message: M = serde_json::from_slice(message.get_payload_bytes())?;
        Ok(Some(message))
    }
}
