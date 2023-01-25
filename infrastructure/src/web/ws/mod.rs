pub mod broker;
pub mod message;
pub mod session;

use self::session::WsSession;
use crate::web::ws::broker::Broker;
use actix::Addr;
use actix_web::{web, HttpRequest, Responder};
use actix_web_actors::ws;
use std::{collections::HashMap, time::Instant};
use thiserror::Error;

/// Start a session actor with the provided session ID, the address of a broker and a callback that
/// populates the session actor's domains. The closure provided for `populate_fn` can utilise the
/// [ws_register!][crate::ws_register] macro.
pub fn start<F>(
    req: &HttpRequest,
    stream: web::Payload,
    session_id: &str,
    broker: Addr<Broker>,
    populate_fn: F,
) -> Result<impl Responder, WsError>
where
    F: FnOnce(&mut WsSession),
{
    let mut session = WsSession {
        id: session_id.to_string(),
        heartbeat: Instant::now(),
        broker,
        callbacks: HashMap::new(),
    };
    populate_fn(&mut session);
    ws::start(session, req, stream).map_err(|e| e.into())
}

#[derive(Debug, Error)]
pub enum WsError {
    #[error("Parsing error: {0}")]
    Parser(#[from] serde_json::Error),
    #[error("Malformed domain: {0}")]
    MalformedDomain(String),
    #[error("Event not implemented: {0}")]
    EventNotImplemented(String),
    #[error("Actix error ocurred: {0}")]
    Actix(#[from] actix_web::Error),
}

#[cfg(test)]
mod tests {
    use super::{message::WsMessage, session::WsSession, WsError};
    use crate::{
        web::ws::broker::{Broker, IssueSync, Subscribe},
        web::ws::message::ActorMessage,
        ws_register,
    };
    use actix::{Actor, Addr, Context, Handler, Message};
    use serde::{Deserialize, Serialize};
    use std::time::Duration;

    struct ActorA {
        count: usize,
    }

    struct ActorB {
        actor_c: Addr<ActorC>,
        count: usize,
    }

    struct ActorC {
        count: usize,
    }

    impl Actor for ActorA {
        type Context = Context<Self>;
    }

    impl Actor for ActorB {
        type Context = Context<Self>;
    }

    impl Actor for ActorC {
        type Context = Context<Self>;
    }

    impl Handler<ActorMessage<TestData>> for ActorA {
        type Result = ();
        fn handle(&mut self, _: ActorMessage<TestData>, _: &mut Self::Context) -> Self::Result {
            self.count += 1;
        }
    }
    // Set up ActorB to send a message to ActorC
    impl Handler<ActorMessage<TestData>> for ActorB {
        type Result = ();
        fn handle(&mut self, msg: ActorMessage<TestData>, _: &mut Self::Context) -> Self::Result {
            self.actor_c.do_send(msg);
            self.count += 1;
        }
    }
    impl Handler<ActorMessage<TestData>> for ActorC {
        type Result = ();
        fn handle(&mut self, _: ActorMessage<TestData>, _: &mut Self::Context) -> Self::Result {
            self.count += 1;
        }
    }

    // Set up ActorB to send a message to ActorC
    impl Handler<ActorMessage<TestData2>> for ActorB {
        type Result = ();
        fn handle(&mut self, _: ActorMessage<TestData2>, _: &mut Self::Context) -> Self::Result {
            self.count += 1;
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Message)]
    #[rtype("()")]
    pub struct Assert(usize);

    impl Handler<Assert> for ActorA {
        type Result = ();
        fn handle(&mut self, msg: Assert, _: &mut Self::Context) -> Self::Result {
            assert_eq!(self.count, msg.0)
        }
    }
    impl Handler<Assert> for ActorB {
        type Result = ();
        fn handle(&mut self, msg: Assert, _: &mut Self::Context) -> Self::Result {
            assert_eq!(self.count, msg.0)
        }
    }
    impl Handler<Assert> for ActorC {
        type Result = ();
        fn handle(&mut self, msg: Assert, _: &mut Self::Context) -> Self::Result {
            assert_eq!(self.count, msg.0)
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Message)]
    #[rtype("()")]
    #[serde(rename_all = "camelCase")]
    pub struct TestData {
        pub a: String,
        pub b: usize,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Message)]
    #[rtype("()")]
    #[serde(rename_all = "camelCase")]
    pub struct TestData2 {
        pub a: String,
        pub b: TestData,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Message)]
    #[rtype("()")]
    #[serde(rename_all = "camelCase", untagged)]
    pub enum TestEnum {
        One(TestData),
        Two(TestData2),
    }

    impl Handler<ActorMessage<TestEnum>> for ActorC {
        type Result = ();
        fn handle(&mut self, msg: ActorMessage<TestEnum>, _: &mut Self::Context) -> Self::Result {
            match msg.into_inner() {
                TestEnum::One(_) => self.count += 10,
                TestEnum::Two(_) => self.count += 20,
            }
        }
    }

    impl Handler<ActorMessage<TestEnum>> for ActorA {
        type Result = ();
        fn handle(&mut self, msg: ActorMessage<TestEnum>, _: &mut Self::Context) -> Self::Result {
            match msg.into_inner() {
                TestEnum::One(_) => self.count += 10,
                TestEnum::Two(_) => self.count += 20,
            }
        }
    }

    #[actix_web::main]
    #[test]
    async fn enums() -> Result<(), WsError> {
        let a1 = ActorA { count: 0 };
        let actor1 = a1.start();

        let a3 = ActorC { count: 0 };
        let actor3 = a3.start();

        let broker = Broker::new("Broker Pitt").start();
        broker.do_send(Subscribe::<ActorMessage<TestEnum>>(
            actor1.clone().recipient(),
        ));

        let mut session = WsSession::new(String::from("Session Van Damme"), broker);

        let json1 = serde_json::to_string(&WsMessage::__mock(TestEnum::One(TestData {
            a: String::from("SoMeThInG"),
            b: 5,
        })))
        .unwrap();

        let json2 = serde_json::to_string(&WsMessage::__mock(TestEnum::Two(TestData2 {
            a: String::from("SoMeThInG"),
            b: TestData {
                a: String::from("hello"),
                b: 10,
            },
        })))
        .unwrap();

        ws_register!(session, "mock", TestEnum, mock);

        let res1 = session.handle_ws_message(json1);
        let res2 = session.handle_ws_message(json2);

        assert!(matches!(res1, Ok(())));
        assert!(matches!(res2, Ok(())));

        tokio::time::sleep(Duration::from_millis(100)).await;

        // Actor 1 is subscribed to both, actor 3 to none
        actor1.send(Assert(30)).await.unwrap();
        actor3.send(Assert(0)).await.unwrap();

        Ok(())
    }

    #[actix_web::main]
    #[test]
    async fn simulate_ws() -> Result<(), WsError> {
        // Init the actors and broker
        let a1 = ActorA { count: 0 };
        let actor1 = a1.start();

        let a3 = ActorC { count: 0 };
        let actor3 = a3.start();

        let a2 = ActorB {
            count: 0,
            actor_c: actor3.clone(),
        };
        let actor2 = a2.start();

        let broker = Broker::new("Broker Pitt").start();
        broker.do_send(Subscribe::<ActorMessage<TestData>>(
            actor1.clone().recipient(),
        ));
        broker.do_send(Subscribe::<ActorMessage<TestData>>(
            actor2.clone().recipient(),
        ));
        broker.do_send(Subscribe::<ActorMessage<TestData2>>(
            actor2.clone().recipient(),
        ));

        let mut session = WsSession::new(String::from("Session Van Damme"), broker);

        ws_register!(session, "mock", TestData, mock);

        // __mock returns a WsMessage with "mock" as its domain
        let json = serde_json::to_string(&WsMessage::__mock(TestData {
            a: String::from("SoMeThInG"),
            b: 5,
        }))
        .unwrap();

        let res = session.handle_ws_message(json);

        assert!(matches!(res, Ok(())));

        // Sleep so actors have time to receive messages
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Actor 1 and 2 increment their count via broker, Actor 2 sends
        // message to Actor 3 on receiving it so all counters should be 1
        actor1.send(Assert(1)).await.unwrap();
        actor2.send(Assert(1)).await.unwrap();
        actor3.send(Assert(1)).await.unwrap();

        ws_register!(session, "hello", TestData2, hello);

        // WS Message with "hello" domain
        let json = serde_json::to_string(&WsMessage {
            domain: "hello".to_string(),
            data: (TestData2 {
                a: String::from("SoMeThInG"),
                b: TestData {
                    a: "world".to_string(),
                    b: 1,
                },
            }),
        })
        .unwrap();

        let res = session.handle_ws_message(json);

        assert!(matches!(res, Ok(())));

        tokio::time::sleep(Duration::from_millis(100)).await;

        // Actor 2 is the only one subscribed to TestData2
        actor1.send(Assert(1)).await.unwrap();
        actor2.send(Assert(2)).await.unwrap();
        actor3.send(Assert(1)).await.unwrap();

        let json = r#"{"invalid":"neverGonnaWork"}"#;

        let res = session.handle_ws_message(json.to_string());

        assert!(matches!(res, Err(e) if matches!(e, WsError::MalformedDomain(_))));

        Ok(())
    }
}
