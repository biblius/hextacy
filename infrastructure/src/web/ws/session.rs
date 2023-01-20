//! Actor that manages client JSON messages
use super::message::WsMessage;
use super::WsError;
use actix::prelude::*;
use actix_web_actors::ws;
use colored::Colorize;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};
use tracing::{debug, info, warn};

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// The session instance. Gets created each time a client connects. `WsSession` is different from
/// actors in the system in that its context is actix's `WebsocketContext`. It is responsible for
/// receiving signals from the client and distributing them to other actors in the system as well as
/// sending messages back to the client.
#[derive(Debug)]
pub struct WsSession<T>
where
    T: 'static + Serialize + DeserializeOwned + Send + Sync + Debug + Message,
    <T as Message>::Result: Send,
{
    /// Unique session identifier for this actor
    pub id: &'static str,
    /// The timestamp of the last ping received from the client
    pub heartbeat: Instant,
    /// Contains adresses of other actors in the system
    pub address_book: HashMap<&'static str, Recipient<T>>,
}

impl<T> Actor for WsSession<T>
where
    T: 'static + Serialize + DeserializeOwned + Send + Sync + Debug + Message,
    <T as Message>::Result: Send,
{
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, context: &mut Self::Context) {
        // Start the heartbeat process on session start.
        self.hb(context);
        info!("{}{:?}", "Started session actor with id: ".green(), self.id);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        info!("{}{:?}", "Stopping session actor with id: ".red(), self.id);
        Running::Stop
    }
}

impl<T> WsSession<T>
where
    T: 'static + Serialize + DeserializeOwned + Send + Sync + Debug + Message,
    <T as Message>::Result: Send,
{
    /// A ping message gets sent every `HEARTBEAT_INTERVAL` seconds,
    /// if a pong isn't received for `CLIENT_TIMEOUT` seconds, drop the connection i.e. stop the context
    fn hb(&self, context: &mut ws::WebsocketContext<Self>) {
        context.run_interval(HEARTBEAT_INTERVAL, |actor, context| {
            if Instant::now().duration_since(actor.heartbeat) > CLIENT_TIMEOUT {
                warn!(
                    "{}",
                    "Websocket client heartbeat failed, disconnecting".red()
                );
                context.stop();
                return;
            }
            context.ping(b"");
        });
    }

    /// Registers an actor's address in this actor's address book.
    pub fn register_addr(&mut self, service_id: &'static str, rec: Recipient<T>) {
        let _ = self.address_book.insert(service_id, rec);
    }

    /// Processes the signal retreived from the client and distributes it to the system
    /// based on the `to` parameter of the signal.
    pub fn handle_ws_message(&mut self, json: String) -> Result<(), WsError> {
        let sig = WsMessage::<T>::from_json(&json)?;
        debug!("{} {:?}", "WsSession -- parsed signal:".blue(), sig);
        if let Some(recipient) = self.address_book.get(sig.domain.as_str()) {
            recipient.do_send(sig.data);
        } else {
            warn!("Invalid domain received: {}", sig.domain)
        }
        Ok(())
    }
}
