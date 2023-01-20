//! Contains the trait `Broadcast` and the `Broker`'s signals.

use actix::{Message, Recipient};

/// A simple wrapper around an actix `Message` that binds the implementing item
/// to be safe to send and clone. Used so we don't have to type out all the
/// trait bounds all the time.
pub trait Broadcast: Message<Result = ()> + Send + Clone + 'static {}
impl<M> Broadcast for M where M: Message<Result = ()> + Send + Clone + 'static {}

/// Issues a synchronous message to all of its subscribers. The broker receiving
/// this message will issue it and await the result of each one.
#[derive(Message)]
#[rtype(result = "()")]
pub struct IssueSync<S: Broadcast>(S);
impl<S: Broadcast> IssueSync<S> {
    pub fn new(s: S) -> Self {
        Self(s)
    }
    pub fn get_inner(&self) -> &S {
        &self.0
    }
    pub fn get_inner_mut(&mut self) -> &mut S {
        &mut self.0
    }
}

/// Issues an asynchronous message to all of its subscribers. The broker receiving
/// this message will first issue it with `try_send` and if that fails for some reason
/// it will issue it with `do_send`.
#[derive(Message)]
#[rtype(result = "()")]
pub struct IssueAsync<S: Broadcast>(S);
impl<S: Broadcast> IssueAsync<S> {
    pub fn new(s: S) -> Self {
        Self(s)
    }
    pub fn get_inner(&self) -> &S {
        &self.0
    }
    pub fn get_inner_mut(&mut self) -> &mut S {
        &mut self.0
    }
}

/// Used by other actors to subscribe to brokers who are already running.
#[derive(Message)]
#[rtype(result = "()")]
pub struct Subscribe<S: Broadcast>(Recipient<S>);
impl<S: Broadcast> Subscribe<S> {
    pub fn new(s: Recipient<S>) -> Self {
        Self(s)
    }
    pub fn clone_inner(&self) -> Recipient<S> {
        self.0.clone()
    }
}
