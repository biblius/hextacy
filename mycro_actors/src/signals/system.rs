use chrono::Local;
use core::fmt::Debug;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
/// The signal used for service communication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal<T> {
    id: String,
    to: Option<String>,
    timestamp: chrono::DateTime<Local>,
    src: String,
    data: T,
}

impl<T> actix::Message for Signal<T>
where
    T: 'static + Debug,
{
    type Result = ();
}

impl<T> Signal<T> {
    /// Creates a new signal and sets its metadata (`id` and `timestamp`).
    pub fn new(src: &str, data: T, to: Option<&str>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            to: to.map(|to| to.to_string()),
            timestamp: chrono::Local::now(),
            data,
            src: src.to_string(),
        }
    }

    /// Return the signal id
    pub fn id(&self) -> &str {
        &self.id
    }
    /// Returns the signal's data
    pub fn data(&self) -> &T {
        &self.data
    }
    /// Returns a mutable ref to the signal's data
    pub fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }
    /// Returns the signal's timestamp
    pub fn ts(&self) -> &chrono::DateTime<Local> {
        &self.timestamp
    }
    /// Returns the id of the service this signal was intended for, if any
    pub fn to(&self) -> &Option<String> {
        &self.to
    }
    /// Returns the id of the service the signal was generated
    pub fn src_id(&self) -> &str {
        &self.src
    }

    /// Testing purposes
    pub fn __mock(data: T) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            to: None,
            timestamp: chrono::Local::now(),
            data,
            src: "Signal::__mock()".to_string(),
        }
    }
}
