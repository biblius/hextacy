//! Actor that manages client JSON messages
use crate::ws_signal::WsSignal;

use super::ws_error::WsError;
use actix::prelude::*;
use actix_web_actors::ws;
use colored::Colorize;
use mycro_core::signal::Signal;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};
use tracing::{debug, error, info, warn};

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// The session instance. Gets created each time a client connects. `WsSession` is different from
/// actors in the system in that its context is actix's `WebsocketContext`. It is responsible for
/// receiving signls from the client and distributing them to other actors in the system.
#[derive(Debug)]
pub struct WsSession<T: 'static + Serialize + DeserializeOwned + Send + Sync + Debug> {
    /// Unique session identifier for this actor
    pub id: &'static str,
    /// The timestamp of the last ping received from the client
    pub heartbeat: Instant,
    pub address_book: HashMap<&'static str, Recipient<Signal<T>>>,
}

impl<T> WsSession<T>
where
    T: 'static + Serialize + DeserializeOwned + Send + Sync + Debug + Clone,
{
    /// A ping message gets sent every `HEARTBEAT_INTERVAL` seconds,
    /// if a pong isn't received for `CLIENT_TIMEOUT` seconds, drop the connection i.e. stop the context
    fn hb(&self, context: &mut ws::WebsocketContext<Self>) {
        context.run_interval(HEARTBEAT_INTERVAL, |actor, context| {
            // Check if the duration is greater than the timeout
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

    /// Broadcasts the given signal to each actor in this actor's address book. Awaits the response
    /// of each message sent.
    pub fn broadcast(&mut self, signal: Signal<T>, ctx: &mut ws::WebsocketContext<Self>) {
        debug!("{} - broadcasting signal: {:?}", self.id, signal);
        for addr in self.address_book.values() {
            addr.send(signal.clone())
                .into_actor(self)
                .then(|res, act, _| {
                    match res {
                        Ok(_) => {
                            debug!("{} - succesfully sent a message", act.id)
                        }
                        Err(_) => {
                            error!("{} - an error occurred while sending a message", act.id)
                        }
                    }
                    fut::ready(())
                })
                .wait(ctx)
        }
    }

    /// Registers an actor's address in this actor's address book.
    pub fn register_addr(&mut self, service_id: &'static str, rec: Recipient<Signal<T>>) {
        let _ = self.address_book.insert(service_id, rec);
    }

    /// Processes the signal retreived from the client and distributes it to the system
    /// based on the `to` parameter of the signal.
    pub fn handle_ws_signal(
        &mut self,
        json: String,
        ctx: &mut ws::WebsocketContext<Self>,
    ) -> Result<(), WsError> {
        let sig: Signal<T> = WsSignal::from_json(&json)?.into();
        debug!("{} {:?}", "WsSession -- parsed signal:".blue(), sig);
        if let Some(to) = sig.to() {
            self.address_book
                .get(&to[..])
                .expect("Signal recipient not registered in address book")
                .do_send(sig);
        } else {
            self.broadcast(sig, ctx);
        }

        Ok(())
    }
}

impl<T> Actor for WsSession<T>
where
    T: 'static + Serialize + DeserializeOwned + Send + Sync + Debug + Clone,
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
