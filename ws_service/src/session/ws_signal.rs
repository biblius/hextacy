use crate::signal::{Signal, SignalError};
use actix::Message;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Debug;
use tracing::error;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsSignal<'a, T> {
    s_type: &'a str,
    to: Option<&'a str>,
    data: T,
}
impl<'a, T: Serialize + Deserialize<'a>> WsSignal<'a, T> {
    pub fn new(s_type: &'a str, to: Option<&'a str>, data: T) -> Self {
        Self { s_type, to, data }
    }

    pub fn to_json(&self) -> Result<RawJson, SignalError> {
        let s = serde_json::to_string(&self).map_err(|e| {
            error!("An error occurred while serializing to JSON: {}", e);
            SignalError::Serde(e)
        })?;
        Ok(RawJson(s))
    }

    pub fn from_json(json: &'a str) -> Result<WsSignal<'a, T>, SignalError> {
        let s = serde_json::from_str::<WsSignal<'a, T>>(json).map_err(|e| {
            error!("An error occurred while deserializing to WsSignal: {}", e);
            SignalError::Serde(e)
        })?;
        Ok(s)
    }

    /// Testing purposes. Generates a `WsSignal` with `s_type: "mock"`, `to: None` and the given data
    pub fn __mock(data: T) -> Self {
        Self {
            s_type: "mock",
            to: None,
            data,
        }
    }
}

impl<T: Debug + Serialize + DeserializeOwned> From<WsSignal<'_, T>> for Signal<T> {
    fn from(ws_sig: WsSignal<T>) -> Self {
        Signal::new("ws_service", ws_sig.data, ws_sig.to)
    }
}
