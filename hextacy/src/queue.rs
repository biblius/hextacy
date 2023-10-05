//! Provides the basic interfaces for interacting with message brokers. Current concrete implementations only support JSON out of the box.
//! See [the adapters module][crate::adapters::queue] for examples on how to implement the [Producer] and [Consumer] traits.
//!
//! The traits are designed to work on enums, meaning you want to implement the [QueueHandler]
//! with the `M` as an enum.

use async_trait::async_trait;
use serde::Serialize;
use std::{fmt::Display, marker::PhantomData};
use thiserror::Error;
use tokio::sync::oneshot::{self, Receiver, Sender};
use tracing::{debug, error, warn};

/// Implement on structs that need to handle messages.
#[async_trait]
pub trait QueueHandler<M>
where
    M: Send + 'static,
{
    type Error: Display + Send;
    async fn handle(&mut self, message: M) -> Result<(), Self::Error>;
}

/// Implement on structs that need to publish messages.
#[async_trait]
pub trait Producer {
    type Error: Display;
    async fn publish<M>(&mut self, message: M) -> Result<(), Self::Error>
    where
        M: Serialize + Send + Sync + 'static;
}

/// Implemented on concrete queue consumers. Check out the `adapters` module for
/// concrete implementations.
#[async_trait]
pub trait Consumer<M>: Sized + Send + 'static
where
    M: Send + 'static,
{
    type Error: Display + Send;

    /// Poll this consumer's queue for an available message. This function should
    /// also be responsible for deserializing it, if necessary.
    ///
    /// When this method returns `Ok(None)` it means the consumer stream is closed
    /// and the whole consumer runtime is dropped.
    async fn poll_queue(&mut self) -> Result<Option<M>, Self::Error>;

    /// Starts this consumer's loop in the tokio runtime and returns a handle for sending a stop signal for graceful shutdown.
    fn start(self, handler: impl QueueHandler<M> + Send + 'static) -> Sender<()> {
        let (tx, rx) = oneshot::channel();
        tokio::spawn(ConsumerRuntime::new(self, handler, rx).run());
        tx
    }
}

/// A runtime for consumers with a stop channel. The sending end is obtained from calling [Consumer::start].
struct ConsumerRuntime<C, M, H> {
    consumer: C,
    handler: H,
    rx: Receiver<()>,
    _m: PhantomData<M>,
}

impl<C, M, H> ConsumerRuntime<C, M, H> {
    fn new(consumer: C, handler: H, rx: Receiver<()>) -> Self {
        Self {
            consumer,
            handler,
            rx,
            _m: PhantomData,
        }
    }
}

impl<C, M, H> ConsumerRuntime<C, M, H>
where
    H: QueueHandler<M>,
    C: Consumer<M> + Send,
    M: Send + 'static,
    C::Error: Send,
{
    async fn run(mut self) -> Result<(), C::Error> {
        let mut logged = false;
        loop {
            let message: M = match self.consumer.poll_queue().await {
                Ok(Some(msg)) => msg,
                Ok(None) => {
                    debug!("Consumer stream ended");
                    return Ok(());
                }
                Err(e) => {
                    error!("Error occurred while polling queue: {e}");
                    continue;
                }
            };

            if let Err(e) = self.handler.handle(message).await {
                error!("Error occurred while handling message: {e}")
            }

            match self.rx.try_recv() {
                Ok(_) => return Ok(()),
                Err(e) => match e {
                    oneshot::error::TryRecvError::Empty => continue,
                    oneshot::error::TryRecvError::Closed => {
                        if !logged {
                            logged = true;
                            warn!("Consumer handle dropped! The consumer will keep processing messages, but there is no way to shut it down without exiting the process.")
                        }
                    }
                },
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum QueueError<E: Display> {
    #[error("serde: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("driver: {0}")]
    Driver(E),
}
