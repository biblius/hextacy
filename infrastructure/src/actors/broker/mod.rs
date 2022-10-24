pub mod signals;

use actix::{prelude::*, Recipient};
use actix::{Actor, Context, Handler};
use colored::Colorize;
use signals::{Broadcast, IssueAsync, IssueSync, Subscribe};
use std::any::Any;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::{any::TypeId, collections::HashMap};
use tracing::{debug, trace};

/// An actor used to broadcast signals to any actors subscribed to them.
/// The phantom data is required for satisfying trait bounds on the handler impls.
#[derive(Debug)]
pub struct Broker<T> {
    id: &'static str,
    pub subs: HashMap<TypeId, Vec<Box<dyn Any>>>,
    __p: PhantomData<T>,
}

#[derive(Debug)]
pub struct DefaultBroker;

impl<T: Unpin + 'static> Actor for Broker<T> {
    type Context = Context<Self>;
    fn started(&mut self, _: &mut Self::Context) {
        debug!("Sarted {}", self.id);
    }
}

impl<T> Broker<T> {
    pub fn new(id: &'static str) -> Self {
        Self {
            id,
            subs: HashMap::new(),
            __p: PhantomData,
        }
    }

    /// Adds an actor to this brokers subscribtion list for the given signal.
    pub fn add_sub<S: Broadcast>(&mut self, sub: Recipient<S>) {
        let s_type = TypeId::of::<S>();
        trace!("Broker adding sub for {:?}", s_type);
        let boxed_sub = Box::new(sub);
        if let Some(subs) = self.subs.get_mut(&s_type) {
            subs.push(boxed_sub);
            return;
        }
        self.subs.insert(s_type, vec![boxed_sub]);
    }

    /// Used when broadcasting a message. Drains this broker's subscriptions for the signal
    /// and returns them, if any
    fn take_subs<S: Broadcast>(&mut self) -> Option<(TypeId, Vec<Recipient<S>>)> {
        let id = TypeId::of::<S>();
        let subs = self.subs.get_mut(&id)?;

        let subs = subs
            .drain(..)
            .filter_map(|rec| {
                if let Ok(rec) = rec.downcast::<Recipient<S>>() {
                    Some(rec)
                } else {
                    None
                }
            })
            .map(|rec| *rec)
            .collect();

        trace!("Broker -- take_subs() -- id: {:?} -- subs: {:?}", id, subs);

        Some((id, subs))
    }
}

impl<T, S> Handler<IssueAsync<S>> for Broker<T>
where
    T: 'static + Unpin + Debug,
    S: Debug + Broadcast,
{
    type Result = ();

    fn handle(&mut self, msg: IssueAsync<S>, ctx: &mut Self::Context) -> Self::Result {
        let signal = msg.get_inner();

        trace!("{} -- received IssueAsync for: {:?}", self.id, signal);

        if let Some((id, mut subs)) = self.take_subs() {
            trace!("{}{}{:?}", self.id, " - Issuing async : ".purple(), id);

            subs.drain(..).for_each(|rec| {
                rec.send(signal.clone())
                    .into_actor(self)
                    .map(move |_, act, _| act.add_sub(rec))
                    .wait(ctx)
            })
        }
    }
}

impl<T, S> Handler<IssueSync<S>> for Broker<T>
where
    T: 'static + Unpin + Debug,
    S: Broadcast + Debug,
{
    type Result = ();

    fn handle(&mut self, msg: IssueSync<S>, _ctx: &mut Self::Context) -> Self::Result {
        let signal = msg.get_inner();

        trace!("{} -- received IssueSync for: {:?}", self.id, signal);

        if let Some((id, mut subs)) = self.take_subs() {
            trace!("{}{}{:?}", self.id, " - Issuing sync : ".purple(), id);

            subs.drain(..)
                .for_each(|rec| match rec.try_send(signal.clone()) {
                    Ok(_) => self.add_sub(rec),
                    Err(SendError::Full(_)) => {
                        rec.do_send(signal.clone());
                        self.add_sub(rec);
                    }
                    Err(_) => (),
                })
        }
    }
}

impl<T, S> Handler<Subscribe<S>> for Broker<T>
where
    T: 'static + Unpin + Debug,
    S: Broadcast + Debug,
{
    type Result = ();

    fn handle(&mut self, msg: Subscribe<S>, _: &mut Self::Context) -> Self::Result {
        self.add_sub::<S>(msg.clone_inner());
    }
}
