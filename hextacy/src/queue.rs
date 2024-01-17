//! Provides the basic interfaces for interacting with message brokers. Current concrete implementations only support JSON out of the box.
//! See [the adapters module][crate::adapters::queue] for examples on how to implement the [Producer] and [Consumer] traits.
//!
//! The traits are designed to work on enums, meaning you want to implement the [QueueHandler]
//! with the `M` as an enum.

use serde::Serialize;
use std::error::Error;
use std::{fmt::Display, marker::PhantomData};
use tokio::sync::oneshot::{self, Receiver, Sender};
use tracing::{debug, error, warn};

/// Implement on structs that need to handle messages.
pub trait QueueHandler<M>
where
    M: Send + 'static,
{
    type Error: Display + Send;
    fn handle(
        &mut self,
        message: M,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send;
}

/// Implement on structs that need to publish messages.
pub trait Producer {
    fn publish<M>(&self, message: M) -> impl std::future::Future<Output = Result<(), QueueError>>
    where
        M: Serialize + Send + Sync + 'static;
}

/// Implemented on concrete queue consumers. Check out the `adapters` module for
/// concrete implementations.
pub trait Consumer<M>: Sized + Send + 'static
where
    M: Send + 'static,
{
    /// Poll this consumer's queue for an available message. This function should
    /// also be responsible for deserializing it, if necessary.
    ///
    /// When this method returns `Ok(None)` it means the consumer stream is closed
    /// and the whole consumer runtime is dropped.
    fn poll_queue(
        &mut self,
    ) -> impl std::future::Future<Output = Result<Option<M>, QueueError>> + Send;

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
    H: QueueHandler<M> + Send,
    C: Consumer<M> + Send,
    M: Send + 'static,
{
    async fn run(mut self) -> Result<(), QueueError> {
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

#[derive(Debug)]
pub enum QueueError {
    Serde(serde_json::Error),
    Driver(Box<dyn Error + Send>),
}

impl Display for QueueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueueError::Serde(e) => write!(f, "{e}"),
            QueueError::Driver(e) => write!(f, "{e}"),
        }
    }
}

impl From<serde_json::Error> for QueueError {
    fn from(value: serde_json::Error) -> Self {
        Self::Serde(value)
    }
}
