use actix::Message;
use chrono::Local;
use core::fmt::Debug;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tracing::error;
use uuid::Uuid;
/// The system signal used for service communication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal<T> {
    id: String,
    to: Option<String>,
    timestamp: chrono::DateTime<Local>,
    src: String,
    data: T,
}

impl<'de, T> actix::Message for Signal<T>
where
    T: 'static + Debug + Serialize + Deserialize<'de>,
{
    type Result = ();
}

impl<T> Signal<T>
where
    T: Debug + Serialize + DeserializeOwned,
{
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

    /// Transforms this signal to a `RawJson` ready to be sent to the client through `WsSession`.
    pub fn to_json(&self) -> Result<RawJson, SignalError> {
        let s = serde_json::to_string(&self).map_err(|e| {
            error!("An error occurred while serializing to JSON: {}", e);
            SignalError::Serde(e)
        })?;
        Ok(RawJson(s))
    }

    /// If at any point we already have a fully typed signal in json, this method can be used to
    /// transform it to a system ready signal.
    pub fn from_json(json: &str) -> Result<Self, SignalError> {
        let s = serde_json::from_str::<Signal<T>>(json).map_err(|e| {
            error!("An error occurred while serializing to JSON: {}", e);
            SignalError::Serde(e)
        })?;
        Ok(s)
    }
}

#[derive(Debug)]
pub enum SignalError {
    Serde(serde_json::Error),
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct RawJson(pub String);

impl RawJson {
    pub fn to_inner(&self) -> String {
        self.0.clone()
    }

    pub fn get_inner(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Serialize, Deserialize, Clone)]
    struct SampleData {
        lol: String,
        lel: String,
    }

    #[test]
    fn test_parsing() -> Result<(), SignalError> {
        let data = SampleData {
            lol: "lol".to_string(),
            lel: "lel".to_string(),
        };

        let signal = Signal::new("test", data, None);

        assert_eq!(signal.src_id(), "test");
        assert_eq!(signal.data().lol, "lol");
        assert_eq!(signal.data().lel, "lel");

        let json = signal.to_json()?;

        let signal = Signal::<SampleData>::from_json(json.get_inner())?;

        assert_eq!(signal.src_id(), "test");
        assert_eq!(signal.data().lol, "lol");
        assert_eq!(signal.data().lel, "lel");

        Ok(())
    }
}
