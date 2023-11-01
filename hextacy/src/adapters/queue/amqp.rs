use async_trait::async_trait;
use futures_util::StreamExt;
use lapin::{
    options::{BasicConsumeOptions, BasicPublishOptions, QueueDeclareOptions},
    types::FieldTable,
    BasicProperties, Connection, ConnectionProperties,
};
use serde::{de::DeserializeOwned, Serialize};
use std::{fmt::Debug, sync::Arc};

use crate::queue::{Consumer, Producer, QueueError};

#[derive(Clone)]
pub struct AmqpDriver {
    pub conn: Arc<Connection>,
}

impl Debug for AmqpDriver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AmqpDriver")
            .field("conn", &"{ .. }")
            .finish()
    }
}

impl AmqpDriver {
    pub async fn new(url: &str) -> Self {
        let conn = Connection::connect(url, ConnectionProperties::default())
            .await
            .expect("Could not establish connection to queue");
        Self {
            conn: Arc::new(conn),
        }
    }

    #[inline]
    /// Calls [create_channel][lapin::Channel] on the connection and sets up a producer for the queue.
    pub async fn publisher_default(
        &self,
        queue: &str,
        exchange: Option<&str>,
    ) -> Result<AmqpPublisher, lapin::Error> {
        let channel = self.conn.create_channel().await?;
        channel
            .queue_declare(queue, QueueDeclareOptions::default(), FieldTable::default())
            .await?;
        Ok(AmqpPublisher {
            queue: queue.to_string(),
            exchange: exchange.map(ToOwned::to_owned),
            channel,
        })
    }

    #[inline]
    /// Calls [create_channel][lapin::Channel] on the connection and sets up a consumer for the queue.
    pub async fn consumer_default(
        &self,
        queue: &str,
        tag: &str,
    ) -> Result<lapin::Consumer, lapin::Error> {
        let channel = self.conn.create_channel().await?;
        let consumer = channel
            .basic_consume(
                queue,
                tag,
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await?;
        Ok(consumer)
    }
}

#[derive(Debug)]
pub struct AmqpPublisher {
    queue: String,
    exchange: Option<String>,
    channel: lapin::Channel,
}

#[async_trait]
impl Producer for AmqpPublisher {
    async fn publish<M>(&self, message: M) -> Result<(), QueueError>
    where
        M: Serialize + Send + Sync + 'static,
    {
        self.channel
            .basic_publish(
                self.exchange.as_deref().unwrap_or_default(),
                &self.queue,
                BasicPublishOptions::default(),
                serde_json::to_string(&message)?.as_bytes(),
                BasicProperties::default(),
            )
            .await
            .map(|_| ())
            .map_err(|e| QueueError::Driver(Box::new(e)))
    }
}

#[async_trait]
impl<M> Consumer<M> for lapin::Consumer
where
    M: DeserializeOwned + Send + Sync + 'static,
{
    async fn poll_queue(&mut self) -> Result<Option<M>, QueueError> {
        let Some(msg) = self.next().await else {
            return Ok(None);
        };
        let message = msg.map_err(|e| QueueError::Driver(Box::new(e)))?;
        let message: M = serde_json::from_slice(&message.data)?;
        Ok(Some(message))
    }
}
