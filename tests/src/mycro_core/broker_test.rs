use actix::{Actor, Context, Handler, System};
use colored::Colorize;
use env_logger::{fmt::Color, Env};
use mycro_core::{
    broker::{
        broker::{Broker, DefaultBroker},
        signals::{IssueAsync, IssueSync},
    },
    Signal,
};
use std::io::Write;
use std::{env, sync::mpsc};
use tracing::debug;

pub fn broker_test(tracing_level: &str) {
    env::set_var("TRACING_LEVEL", tracing_level);
    env_logger::Builder::from_env(Env::default().filter("TRACING_LEVEL"))
        .format(|buf, record| {
            let mut style = buf.style();
            style.set_color(Color::Cyan);
            writeln!(buf, "{}", style.value(record.args()))
        })
        .init();

    let sys = System::new();
    let mut broker = Broker::<DefaultBroker>::new("TEST_BROKER");

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
        addr_b.send(IssueSync(sig.clone())).await.unwrap();
        addr_b.send(IssueSync(sig_o.clone())).await.unwrap();
        addr_b.send(IssueSync(sig_s.clone())).await.unwrap();

        addr_b.send(IssueAsync(sig)).await.unwrap();
        addr_b.send(IssueAsync(sig_o)).await.unwrap();
        addr_b.send(IssueAsync(sig_s)).await.unwrap();

        tx.send(42).unwrap();
    };

    let _ = sys.block_on(exec);

    let num = rx.recv().unwrap();

    assert_eq!(42, num);
}
#[derive(Debug, Clone)]
struct SampleData;
#[derive(Debug, Clone)]
struct Sample;
#[derive(Debug, Clone)]
struct OtherSample;

/// Test Actor
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
        debug!("{}{}{:?}", self.id, " -- received signal :".purple(), sig);
    }
}
impl Handler<Signal<Sample>> for TestActor {
    type Result = ();
    fn handle(&mut self, sig: Signal<Sample>, _: &mut Self::Context) -> Self::Result {
        debug!("{}{}{:?}", self.id, " -- received signal :".purple(), sig);
    }
}
impl Handler<Signal<OtherSample>> for TestActor {
    type Result = ();
    fn handle(&mut self, sig: Signal<OtherSample>, _: &mut Self::Context) -> Self::Result {
        debug!("{}{}{:?}", self.id, " -- received signal :".purple(), sig);
    }
}
