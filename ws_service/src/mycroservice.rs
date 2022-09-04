use std::fmt::Debug;

use actix::Recipient;
use serde::{de::DeserializeOwned, Serialize};

use crate::signal::Signal;

pub trait MycroService<T: Debug + Serialize + DeserializeOwned + Send + Sync> {
    fn id(&self) -> String;
    fn register_addr(&mut self, id: &str, rec: Recipient<Signal<T>>);
    fn send(&self, rec_id: &str, msg: &Signal<T>);
    fn broadcast(&self, msg: &Signal<T>);
}
