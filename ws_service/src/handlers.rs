use super::{actor::WsSession, signals::raw_json::RawJson};
use actix::prelude::*;
use actix_web_actors::ws;
use serde::{de::DeserializeOwned, Serialize};
use std::{fmt::Debug, time::Instant};
use tracing::{error, trace, warn};

/// The session actor implements a handler only for the `RawJson` message type.
/// Any message sent to this actor will get sent back to the web socket context as JSON data.
impl<T> Handler<RawJson> for WsSession<T>
where
    T: 'static + Serialize + DeserializeOwned + Send + Sync + Debug + Clone,
{
    type Result = ();
    fn handle(&mut self, msg: RawJson, context: &mut Self::Context) {
        context.text(msg.to_inner());
    }
}

/// A handler for the websocket data stream. All the signals received from the client will land
/// on the `ws::Message::Text` where the actor then handles the signal appropriately.
impl<T> StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession<T>
where
    T: 'static + Serialize + DeserializeOwned + Send + Sync + Debug + Clone,
{
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, context: &mut Self::Context) {
        let msg = match msg {
            Ok(msg) => msg,
            Err(_) => {
                context.stop();
                return;
            }
        };
        match msg {
            ws::Message::Ping(msg) => {
                self.heartbeat = Instant::now();
                trace!("Session: {} got ping, sending pong", self.id,);
                context.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.heartbeat = Instant::now();
                trace!("Session: {} got pong", self.id,)
            }
            ws::Message::Text(text) => {
                self.handle_ws_signal(text.to_string(), context)
                    .map_err(|e| error!("An error occurred while processing a message: {e}"))
                    .unwrap();
            }
            ws::Message::Binary(_) => warn!("Unexpected binary"),
            ws::Message::Close(reason) => {
                context.close(reason);
                context.stop();
            }
            ws::Message::Continuation(_) => {
                context.stop();
            }
            ws::Message::Nop => (),
        }
    }
}
