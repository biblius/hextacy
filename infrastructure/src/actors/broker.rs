//! Contains the trait `Broadcast` and the `Broker`'s messages.

use actix::Message;
use actix::{prelude::*, Recipient};
use actix::{Actor, Context, Handler};
use colored::Colorize;
use std::any::Any;
use std::fmt::Debug;
use std::{any::TypeId, collections::HashMap};
use tracing::{debug, trace};

/// An actor used to broadcast messages to any actors subscribed to them.
#[derive(Debug)]
pub struct Broker {
    id: &'static str,
    pub subs: HashMap<TypeId, Vec<Box<dyn Any>>>,
}

impl Actor for Broker {
    type Context = Context<Self>;
    fn started(&mut self, _: &mut Self::Context) {
        debug!("Started Broker: {}", self.id);
    }
}

impl Broker {
    pub fn new(id: &'static str) -> Self {
        Self {
            id,
            subs: HashMap::new(),
        }
    }

    /// Adds an actor to this brokers subscribtion list for the given message.
    pub fn add_sub<M: Broadcast>(&mut self, sub: Recipient<M>) {
        let s_type = TypeId::of::<M>();
        trace!("Broker adding sub for {:?}", s_type);
        let boxed_sub = Box::new(sub);
        if let Some(subs) = self.subs.get_mut(&s_type) {
            subs.push(boxed_sub);
            return;
        }
        self.subs.insert(s_type, vec![boxed_sub]);
    }

    /// Used when broadcasting a message. Drains this broker's subscriptions for the message
    /// and returns them, if any
    fn take_subs<M: Broadcast>(&mut self) -> Option<(TypeId, Vec<Recipient<M>>)> {
        let id = TypeId::of::<M>();
        let subs = self.subs.get_mut(&id)?;

        let subs = subs
            .drain(..)
            .filter_map(|rec| {
                if let Ok(rec) = rec.downcast::<Recipient<M>>() {
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

impl<M> Handler<IssueAsync<M>> for Broker
where
    //T: 'static + Unpin + Debug,
    M: Debug + Broadcast,
{
    type Result = ();

    fn handle(&mut self, msg: IssueAsync<M>, ctx: &mut Self::Context) -> Self::Result {
        let message = msg.get_inner();

        trace!("{} -- received IssueAsync for: {:?}", self.id, message);

        if let Some((id, mut subs)) = self.take_subs() {
            trace!("{}{}{:?}", self.id, " - Issuing async : ".purple(), id);

            subs.drain(..).for_each(|rec| {
                rec.send(message.clone())
                    .into_actor(self)
                    .map(move |_, act, _| act.add_sub(rec))
                    .wait(ctx)
            })
        }
    }
}

impl<M> Handler<IssueSync<M>> for Broker
where
    // T: 'static + Unpin + Debug,
    M: Broadcast + Debug,
{
    type Result = ();

    fn handle(&mut self, msg: IssueSync<M>, _ctx: &mut Self::Context) -> Self::Result {
        let message = msg.get_inner();

        trace!("{} -- received IssueSync for: {:?}", self.id, message);

        if let Some((id, mut subs)) = self.take_subs() {
            trace!("{}{}{:?}", self.id, " - Issuing sync : ".purple(), id);

            subs.drain(..)
                .for_each(|rec| match rec.try_send(message.clone()) {
                    Ok(_) => self.add_sub(rec),
                    Err(SendError::Full(_)) => {
                        rec.do_send(message.clone());
                        self.add_sub(rec);
                    }
                    Err(_) => (),
                })
        }
    }
}

impl<M> Handler<Subscribe<M>> for Broker
where
    // T: 'static + Unpin + Debug,
    M: Broadcast + Debug,
{
    type Result = ();

    fn handle(&mut self, msg: Subscribe<M>, _: &mut Self::Context) -> Self::Result {
        self.add_sub::<M>(msg.clone_inner());
    }
}

/// A simple wrapper around an actix `Message` that binds the implementing item
/// to be safe to send and clone. Used so we don't have to type out all the
/// trait bounds all the time.
pub trait Broadcast: Message<Result = ()> + Send + Clone + 'static {}
impl<M> Broadcast for M where M: Message<Result = ()> + Send + Clone + 'static {}

/// Issues a synchronous message to all of its subscribers. The broker receiving
/// this message will issue it and await the result of each one.
#[derive(Message)]
#[rtype(result = "()")]
pub struct IssueSync<M: Broadcast>(pub M);
impl<M: Broadcast> IssueSync<M> {
    pub fn new(s: M) -> Self {
        Self(s)
    }
    pub fn get_inner(&self) -> &M {
        &self.0
    }
}

/// Issues an asynchronous message to all of its subscribers. The broker receiving
/// this message will first issue it with `try_send` and if that fails for some reason
/// it will issue it with `do_send`.
#[derive(Message)]
#[rtype(result = "()")]
pub struct IssueAsync<M: Broadcast>(pub M);
impl<M: Broadcast> IssueAsync<M> {
    pub fn new(s: M) -> Self {
        Self(s)
    }
    pub fn get_inner(&self) -> &M {
        &self.0
    }
}

/// Used by other actors to subscribe to brokers who are already running.
#[derive(Message)]
#[rtype(result = "()")]
pub struct Subscribe<M: Broadcast>(pub Recipient<M>);
impl<M: Broadcast> Subscribe<M> {
    pub fn new(s: Recipient<M>) -> Self {
        Self(s)
    }
    pub fn clone_inner(&self) -> Recipient<M> {
        self.0.clone()
    }
}
