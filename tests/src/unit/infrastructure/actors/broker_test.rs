use actors::actix::{Actor, Context, Handler, System};
use actors::{
    broker::signals::{IssueAsync, IssueSync, Subscribe},
    broker::{Broker, DefaultBroker},
    Signal,
};
use colored::Colorize;
use std::sync::mpsc;
use tracing::{debug, info};

const BROKER_ID: &str = "TEST_BROKER";

pub fn add_sub() {
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

pub fn handle_subscribe() {
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
