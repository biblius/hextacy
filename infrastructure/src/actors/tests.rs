mod broker {
    use super::super::{
        broker::signals::{IssueAsync, IssueSync, Subscribe},
        broker::{Broker, DefaultBroker},
        Signal,
    };
    use actix::{Actor, Context, Handler, System};
    use colored::Colorize;
    use std::sync::mpsc;
    use tracing::{debug, info};

    const BROKER_ID: &str = "TEST_BROKER";

    #[derive(Debug, Clone)]
    struct SampleData;
    #[derive(Debug, Clone)]
    struct Sample;
    #[derive(Debug, Clone)]
    struct OtherSample;

    struct TestActor {
        id: &'static str,
    }

    impl Actor for TestActor {
        type Context = Context<Self>;
        fn started(&mut self, _: &mut Self::Context) {
            debug!("Started {}", self.id,);
        }
    }

    impl Handler<Signal<SampleData>> for TestActor {
        type Result = ();
        fn handle(&mut self, sig: Signal<SampleData>, _: &mut Self::Context) -> Self::Result {
            debug!("{}{}{:?}", self.id, " -- received signal: ".purple(), sig);
        }
    }
    impl Handler<Signal<Sample>> for TestActor {
        type Result = ();
        fn handle(&mut self, sig: Signal<Sample>, _: &mut Self::Context) -> Self::Result {
            debug!("{}{}{:?}", self.id, " -- received signal: ".purple(), sig);
        }
    }
    impl Handler<Signal<OtherSample>> for TestActor {
        type Result = ();
        fn handle(&mut self, sig: Signal<OtherSample>, _: &mut Self::Context) -> Self::Result {
            debug!("{}{}{:?}", self.id, " -- received signal: ".purple(), sig);
        }
    }
    #[test]
    fn add_sub() {
        info!("\n========== TEST - BROKER ADD SUB ==========\n");
        let sys = System::new();
        let mut broker = Broker::<DefaultBroker>::new(BROKER_ID);

        let test_act = TestActor { id: "TEST_ACTOR" };
        let other_act = TestActor { id: "OTHER_ACTOR" };

        let sig = Signal::new("test", SampleData {}, None);
        let sig_s = Signal::new("test", Sample {}, None);
        let sig_o = Signal::new("test", OtherSample {}, None);

        let (tx, rx) = mpsc::channel::<usize>();

        let exec = async move {
            let addr = test_act.start();
            let other_addr = other_act.start();

            broker.add_sub::<Signal<SampleData>>(addr.clone().recipient());
            broker.add_sub::<Signal<SampleData>>(other_addr.clone().recipient());
            broker.add_sub::<Signal<Sample>>(addr.recipient());
            broker.add_sub::<Signal<OtherSample>>(other_addr.recipient());

            // There should be 8 received signal logs in total
            let addr_b = broker.start();
            addr_b.send(IssueSync::new(sig.clone())).await.unwrap();
            addr_b.send(IssueSync::new(sig_o.clone())).await.unwrap();
            addr_b.send(IssueSync::new(sig_s.clone())).await.unwrap();

            addr_b.send(IssueAsync::new(sig)).await.unwrap();
            addr_b.send(IssueAsync::new(sig_o)).await.unwrap();
            addr_b.send(IssueAsync::new(sig_s)).await.unwrap();

            tx.send(42).unwrap();
        };

        sys.block_on(exec);

        let num = rx.recv().unwrap();

        assert_eq!(42, num);
    }
    #[test]
    fn handle_subscribe() {
        info!("\n========== TEST - BROKER HANDLE SUBSCRIBE ==========\n");
        let sys = System::new();
        let broker = Broker::<DefaultBroker>::new(BROKER_ID);

        let test_act = TestActor { id: "TEST_ACTOR" };
        let other_act = TestActor { id: "OTHER_ACTOR" };

        let sig = Signal::new("test", SampleData {}, None);
        let sig_s = Signal::new("test", Sample {}, None);

        let exec = async move {
            let b_addr = broker.start();

            let addr = test_act.start();
            let addr_o = other_act.start();

            b_addr
                .send(Subscribe::<Signal<SampleData>>::new(addr.recipient()))
                .await
                .unwrap();
            b_addr
                .send(Subscribe::<Signal<Sample>>::new(addr_o.recipient()))
                .await
                .unwrap();
            b_addr.send(IssueSync::new(sig)).await.unwrap();
            b_addr.send(IssueSync::new(sig_s)).await.unwrap();
        };

        sys.block_on(exec);
    }
}

mod direct {
    use super::super::Signal;
    use crate::web::websocket::message::WsMessage;
    use actix::{
        fut, Actor, ActorFutureExt, Context, ContextFutureSpawner, Handler, Recipient, System,
        WrapFuture,
    };
    use serde::{de::DeserializeOwned, Deserialize, Serialize};
    use std::fmt::Debug;
    use std::io::Error;
    use std::sync::mpsc;
    use std::{collections::HashMap, marker::PhantomData};
    use tracing::{debug, error, info};

    /// Also tests WsMessage conversion to system message
    #[test]
    fn simple_message_handling() -> Result<(), Error> {
        info!("\n========== TEST - SIMPLE MESSAGE HANDLING ==========\n");
        // Initialize system
        let sys = System::new();

        // Initialize the actor
        let act: TestActor<SampleData> = TestActor {
            id: "TEST_ACTOR",
            addr_book: HashMap::new(),
        };

        // Initialize the signal
        let sig: Signal<SampleData> = WsMessage::__mock(SampleData {
            lol: "lol".to_string(),
            lel: "lel".to_string(),
        })
        .into();

        // Set up channel so we can get back the result on this thread
        let (tx, rx) = mpsc::channel::<usize>();

        // Prepare a future for the system. When the actor starts we send the initialized signal
        // to its address and await the result. The signal goes to the Handler which prints what message
        // it received and returns a unit type. If it's ok we send back a message to the receiver.
        let execution = async move {
            let add = act.start();
            let res = add.send(sig).await;
            if res.is_ok() {
                tx.send(42).unwrap()
            }
        };

        // Block until the future is done
        sys.block_on(execution);

        // Catch the message sent from it
        let num = rx.recv().unwrap();

        assert_eq!(num, 42);

        Ok(())
    }

    /// Initiates an actor and adds another to its address book and tries to send the message
    /// to everyone in the actor's address book without the use of the broker
    #[test]
    fn simple_broadcast() -> Result<(), Error> {
        info!("\n========== TEST - SIMPLE DIRECT BROADCAST ==========\n");
        // Initialize system
        let sys = System::new();

        // Initialize the sending actor and the context it will run in
        let mut act: TestActor<SampleData> = TestActor {
            id: "TEST_ACTOR",
            addr_book: HashMap::new(),
        };
        let ctx = Context::<TestActor<SampleData>>::new();

        // Initialize the receiving actor
        let my_act = MyActor::<SampleData> {
            id: "MY_ACTOR",
            p: PhantomData,
        };

        // Initialize the signal
        let sig: Signal<SampleData> = WsMessage::__mock(SampleData {
            lol: "lmao".to_string(),
            lel: "lmeo".to_string(),
        })
        .into();

        // Init channel for result
        let (tx, rx) = mpsc::channel::<usize>();

        // First starts the receiving actor then registers its address on the sending one. The sending
        // one then broadcasts the signal and awaits the result, if all went well we send a 42 to the channel
        let exec = async move {
            let my_id = my_act.id;
            let my_addr = my_act.start();
            act.register_addr(my_id, my_addr.recipient());
            let addr = ctx.run(act);
            let r = addr.send(sig).await;
            if r.is_ok() {
                tx.send(42).unwrap();
            }
        };

        sys.block_on(exec);

        let num = rx.recv().unwrap();

        assert_eq!(num, 42);

        Ok(())
    }

    #[test]
    fn multi_send() {
        let sys = System::new();
        let act = MyActor::<SampleData> {
            id: "lmeo",
            p: PhantomData,
        };

        // Initialize the signal
        let sig: Signal<SampleData> = WsMessage::__mock(SampleData {
            lol: "lmao".to_string(),
            lel: "lmeo".to_string(),
        })
        .into();

        let exec = async move {
            let addr = act.start();
            let r = addr.send(sig.clone()).await;
            assert!(r.is_ok());
            let r = addr.send(sig).await;
            assert!(r.is_ok());
        };

        sys.block_on(exec);
    }

    /// Mock Actor
    struct MyActor<T> {
        id: &'static str,
        p: PhantomData<T>,
    }

    impl<T> Actor for MyActor<T>
    where
        T: 'static + Serialize + DeserializeOwned + Send + Sync + Debug + Unpin + Clone,
    {
        type Context = Context<Self>;
        fn started(&mut self, _: &mut Self::Context) {
            debug!("{} -- started", self.id);
        }
    }

    impl Handler<Signal<SampleData>> for MyActor<SampleData> {
        type Result = ();

        fn handle(&mut self, mut sig: Signal<SampleData>, _: &mut Self::Context) -> Self::Result {
            sig.data_mut().lol = "Modified lmao".to_string();
            sig.data_mut().lel = "Modified lmeo".to_string();
            debug!("{} -- received signal : {:?}", self.id, sig);
        }
    }

    #[derive(Debug, Serialize, Deserialize, Clone)]
    struct SampleData {
        lol: String,
        lel: String,
    }

    /// Test Actor is wired to broadcast a SampleData signal whenever it receives one
    struct TestActor<T: 'static + Serialize + DeserializeOwned + Send + Sync + Debug + Clone> {
        id: &'static str,
        addr_book: HashMap<&'static str, Recipient<Signal<T>>>,
    }

    impl<T> TestActor<T>
    where
        T: 'static + Serialize + DeserializeOwned + Send + Sync + Debug + Clone + Unpin,
    {
        pub fn register_addr(&mut self, service_id: &'static str, rec: Recipient<Signal<T>>) {
            let _ = self.addr_book.insert(service_id, rec);
        }

        pub fn broadcast(&mut self, signal: Signal<T>, ctx: &mut Context<Self>) {
            debug!("{} -- broadcasting", self.id);
            for addr in self.addr_book.values() {
                addr.send(signal.clone())
                    .into_actor(self)
                    .then(|res, act, _| {
                        match res {
                            Ok(_) => {
                                debug!("{} - succesfully sent signal", act.id)
                            }
                            Err(_) => {
                                error!("{} - an error occurred while sending a signal", act.id)
                            }
                        }
                        fut::ready(())
                    })
                    .wait(ctx)
            }
        }
    }

    impl<T> Actor for TestActor<T>
    where
        T: 'static + Serialize + DeserializeOwned + Send + Sync + Debug + Unpin + Clone,
    {
        type Context = Context<Self>;

        fn started(&mut self, _: &mut Self::Context) {
            debug!("{} -- started", self.id);
        }
    }

    impl Handler<Signal<SampleData>> for TestActor<SampleData> {
        type Result = ();

        fn handle(&mut self, sig: Signal<SampleData>, ctx: &mut Self::Context) -> Self::Result {
            debug!("{} -- received signal : {:?}", self.id, sig);
            self.broadcast(sig, ctx);
        }
    }
}
