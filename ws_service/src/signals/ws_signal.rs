use super::raw_json::RawJson;
use crate::ws_error::WsError;
use mycro_actors::Signal;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use tracing::error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsSignal<'a, T> {
    s_type: &'a str,
    to: Option<&'a str>,
    data: T,
}

impl<'a, T> WsSignal<'a, T>
where
    T: Serialize + Deserialize<'a> + Debug,
{
    pub fn new(s_type: &'a str, to: Option<&'a str>, data: T) -> Self {
        Self { s_type, to, data }
    }

    pub fn to_json(&self) -> Result<RawJson, WsError> {
        let s = serde_json::to_string(&self).map_err(|e| {
            error!("An error occurred while serializing to JSON: {}", e);
            WsError::Serde(e)
        })?;
        Ok(RawJson(s))
    }

    pub fn from_json(json: &'a str) -> Result<WsSignal<'a, T>, WsError> {
        let s = serde_json::from_str::<WsSignal<'a, T>>(json).map_err(|e| {
            error!("An error occurred while deserializing to WsSignal: {}", e);
            WsError::Serde(e)
        })?;
        Ok(s)
    }

    /// Directly deserialize the json to a system ready signal
    pub fn to_system_signal(json: &'a str) -> Result<Signal<T>, WsError> {
        let s = serde_json::from_str::<WsSignal<'a, T>>(json).map_err(|e| {
            error!("An error occurred while serializing to JSON: {}", e);
            WsError::Serde(e)
        })?;
        Ok(s.into())
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

impl<'a, T: Debug + Serialize + Deserialize<'a>> From<WsSignal<'_, T>> for Signal<T> {
    fn from(ws_sig: WsSignal<T>) -> Self {
        Signal::new("ws_service", ws_sig.data, ws_sig.to)
    }
}
