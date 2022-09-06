use actix::{Message, Recipient};

pub trait Broadcast: Message<Result = ()> + Send + Clone + 'static {}
impl<M> Broadcast for M where M: Message<Result = ()> + Send + Clone + 'static {}

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
