//! Actor that manages client JSON messages
use super::message::RawJson;
use super::WsError;
use crate::web::ws::broker::Broker;
use actix::prelude::*;
use actix_web_actors::ws;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Debug;
use std::time::{Duration, Instant};
use tracing::{error, trace};
use tracing::{info, warn};

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// The session instance. Gets created each time a client connects. `WsSession` is different from
/// actors in the system in that its context is actix's `WebsocketContext`. It is responsible for
/// receiving messages from the client and distributing them to other actors in the system as well as
/// sending messages back to the client.
///
/// ### Example
///
/// ```
/// use hextacy::{
///     web::ws::broker::{Broker, IssueSync, Subscribe},
///     web::ws::message::ActorMessage,
///     ws_register,
/// };
/// use actix::{Actor, Addr, Context, Handler, Message};
/// use serde::{Deserialize, Serialize};
/// use std::time::Duration;
///
/// struct ActorA {
///     count: usize,
/// }
///
/// impl Actor for ActorA {
///     type Context = Context<Self>;
/// }
///
/// impl Handler<ActorMessage<TestData>> for ActorA {
///     type Result = ();
///     fn handle(&mut self, _: ActorMessage<TestData>, _: &mut Self::Context) -> Self::Result {
///         self.count += 1;
///     }
/// }
///
///impl Handler<Assert> for ActorA {
///     type Result = ();
///     fn handle(&mut self, msg: Assert, _: &mut Self::Context) -> Self::Result {
///         assert_eq!(self.count, msg.0)
///     }
/// }
///
///
/// #[derive(Debug, Clone, Serialize, Deserialize, Message)]
/// #[rtype("()")]
/// pub struct Assert(usize);
///
/// #[derive(Debug, Clone, Serialize, Deserialize, Message)]
/// #[rtype("()")]
/// #[serde(rename_all = "camelCase")]
/// pub struct TestData {
///     pub a: String,
///     pub b: usize,
/// }
///
/// #[actix_web::main]
/// #[test]
/// async fn simulate_ws() -> Result<(), WsError> {
///     let actor = ActorA { count: 0 };
///     let actor = actor.start();
///
///     // subscribe actor 1 to TestData via the broker
///     let broker = Broker::new("Broker Pitt").start();
///     broker.do_send(Subscribe::<ActorMessage<TestData>>(
///         actor.clone().recipient(),
///     ));
///
///     let mut session = WsSession::new(String::from("Session Van Damme"), broker);
///
///     // Register the 'hello' domain for TestData
///     ws_register!(session, "hello", TestData, hello);
///
///     // WS Message with "hello" domain
///     let json = serde_json::to_string(&WsMessage {
///         domain: "hello".to_string(),
///         data: TestData {
///                 a: "world".to_string(),
///                 b: 1,
///         },
///     })
///     .unwrap();
///
///     let res = session.handle_ws_message(json);
///
///     assert!(matches!(res, Ok(())));
///
///     tokio::time::sleep(Duration::from_millis(100)).await;
///
///     actor.send(Assert(1)).await.unwrap();
///
///     Ok(())
/// }
///```
pub struct WsSession {
    /// Unique session identifier for this actor
    pub id: String,
    /// The timestamp of the last ping received from the client
    pub heartbeat: Instant,
    /// The broker used to propagate obtained messages to other actors via pubsub
    pub broker: Addr<Broker>,
    /// Callback functions mapped to domain names. Messages received from the client
    /// with valid domain names will execute the provided functions, given their data is
    /// in the correct format
    pub callbacks: HashMap<&'static str, WsMessageCallback>,
}

type WsMessageCallback = Box<dyn Fn(&WsSession, String) -> Result<(), WsError>>;

impl WsSession {
    pub fn new(id: String, broker: Addr<Broker>) -> Self {
        Self {
            id,
            heartbeat: Instant::now(),
            broker,
            callbacks: HashMap::new(),
        }
    }

    /// Register a callback for messages received in the specified domain. The [ws_register]
    /// macro makes life easier and you should probably use that.
    pub fn register_handler(&mut self, domain: &'static str, cb: WsMessageCallback) {
        self.callbacks.insert(domain, cb);
    }

    /// Processes the message retreived from the client and distributes it to the system
    /// based on the domain. Gets called whenever the session actor picks up a JSON message
    /// from the client.
    pub fn handle_ws_message(&self, json: String) -> Result<(), WsError> {
        let domain = self.parse_domain(&json)?;
        match self.callbacks.get(domain.as_str()) {
            Some(cb) => {
                cb(self, json)?;
                Ok(())
            }
            None => Err(WsError::EventNotImplemented(domain.to_string())),
        }
    }

    /// A ping message gets sent every `HEARTBEAT_INTERVAL` seconds,
    /// if a pong isn't received for `CLIENT_TIMEOUT` seconds, drop the connection i.e. stop the context
    fn hb(&self, context: &mut ws::WebsocketContext<Self>) {
        context.run_interval(HEARTBEAT_INTERVAL, |actor, context| {
            if Instant::now().duration_since(actor.heartbeat) > CLIENT_TIMEOUT {
                warn!("Websocket client heartbeat failed, disconnecting");
                context.stop();
                return;
            }
            context.ping(b"");
        });
    }

    /// Attempt to parse the 'domain' field of an incoming JSON message.
    fn parse_domain(&self, message: &str) -> Result<String, WsError> {
        let message: Value = serde_json::from_str(message).unwrap();
        let domain = &message["domain"];
        domain.as_str().map_or_else(
            || {
                Err(WsError::MalformedDomain(
                    "Domain malformed, make sure to specify it on the top level of the message"
                        .to_string(),
                ))
            },
            |val| Ok(val.to_string()),
        )
    }
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, context: &mut Self::Context) {
        // Start the heartbeat process on session start.
        self.hb(context);
        info!("Started session actor with id: {}", self.id);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        info!("Stopping session actor with id: {}", self.id);
        Running::Stop
    }
}

/// The session actor implements a handler only for the `RawJson` message type.
/// Any RawJson message sent to this actor will get sent back to the web socket context as JSON data.
impl Handler<RawJson> for WsSession {
    type Result = ();
    fn handle(&mut self, msg: RawJson, context: &mut Self::Context) {
        context.text(msg.into_inner());
    }
}

/// A handler for the websocket data stream. All messages received from the client will land
/// on the `ws::Message::Text` where the actor then handles the message appropriately.
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, context: &mut Self::Context) {
        let msg = match msg {
            Ok(msg) => msg,
            Err(e) => {
                error!("Error occurred while handling WS stream: {e}");
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
                self.handle_ws_message(text.to_string())
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

impl Debug for WsSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WsSession")
            .field("id", &self.id)
            .field("heartbeat", &self.heartbeat)
            .field("broker", &self.broker)
            .finish()
    }
}

#[macro_export]
/// Register a domain on the session actor.
///
/// The first argument must always be an instance of the session actor, not yet started.
///
/// The second argument specifies the domain for the third argument, the data structure
/// expected in the message. JSON messages sent from the client must always specify the domain
/// as it will be used to deserialize the message to the appropriate type. Internally, the message
/// will be transformed into an instance of an [ActorMessage][super::message::ActorMessage], which
/// contains the session ID the message came from.
///
/// Once the message is deserialized into the proper format, it will be broadcast via the session's
/// broker handle, sending it to everyone subscribed to the message type.
///
/// The fourth argument is largely irrelevant and only serves as an internal fn identifier cause I
/// CBA to find another way to create custom fn identifiers.
macro_rules! ws_register {
    ($session:expr, $domain:literal, $data:ty, $id:ident) => {
        fn $id(session: &WsSession, json: String) -> Result<(), WsError> {
            let message = WsMessage::<$data>::to_actor_message(&session.id, &json)?;
            session.broker.do_send(IssueSync(message));
            Ok(())
        }

        $session.register_handler($domain, Box::new($id));
    };
}
