use actix::Message;
use std::fmt::Debug;

pub trait BroadcastSignal: Message<Result = ()> + Send + Clone + 'static + Debug {}
impl<M> BroadcastSignal for M where M: Message<Result = ()> + Send + Clone + 'static + Debug {}

#[derive(Message)]
#[rtype(result = "()")]
pub struct IssueSync<T: BroadcastSignal>(pub T);

impl<T: BroadcastSignal> IssueSync<T> {
    pub fn get_inner(&self) -> &T {
        &self.0
    }
    pub fn get_inner_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct IssueAsync<T: BroadcastSignal>(pub T);

impl<T: BroadcastSignal> IssueAsync<T> {
    pub fn get_inner(&self) -> &T {
        &self.0
    }
    pub fn get_inner_mut(&mut self) -> &mut T {
        &mut self.0
    }
}
