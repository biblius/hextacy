use mycro_core::signal::{RawJson, Signal, SignalError};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Debug;
use tracing::error;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Serialize, Deserialize, Clone)]
    struct SampleData {
        lol: String,
        lel: String,
    }

    #[test]
    fn test_from_ws() {
        let data = SampleData {
            lol: "lol".to_string(),
            lel: "lel".to_string(),
        };
        let ws_signal = WsSignal::new("SampleData", None, data.clone());

        let signal: Signal<SampleData> = ws_signal.into();

        assert_eq!(signal.data().lol, "lol");
        assert_eq!(signal.data().lel, "lel");

        assert!(signal.to().is_none());
    }
}
