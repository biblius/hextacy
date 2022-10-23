use actix::Message;

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

use super::WsError;
use actors::Signal;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use tracing::error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsMessage<'a, T> {
    s_type: &'a str,
    to: Option<&'a str>,
    data: T,
}

impl<'a, T> WsMessage<'a, T>
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

    pub fn from_json(json: &'a str) -> Result<WsMessage<'a, T>, WsError> {
        let s = serde_json::from_str::<WsMessage<'a, T>>(json).map_err(|e| {
            error!("An error occurred while deserializing to WsMessage: {}", e);
            WsError::Serde(e)
        })?;
        Ok(s)
    }

    /// Directly deserialize the json to a system ready signal
    pub fn to_system_signal(json: &'a str) -> Result<Signal<T>, WsError> {
        let s = serde_json::from_str::<WsMessage<'a, T>>(json).map_err(|e| {
            error!("An error occurred while serializing to JSON: {}", e);
            WsError::Serde(e)
        })?;
        Ok(s.into())
    }

    /// Testing purposes. Generates a `WsMessage` with `s_type: "mock"`, `to: None` and the given data
    pub fn __mock(data: T) -> Self {
        Self {
            s_type: "mock",
            to: None,
            data,
        }
    }
}

impl<'a, T: Debug + Serialize + Deserialize<'a>> From<WsMessage<'_, T>> for Signal<T> {
    fn from(ws_sig: WsMessage<T>) -> Self {
        Signal::new("ws_service", ws_sig.data, ws_sig.to)
    }
}

#[cfg(test)]
mod tests {
    use crate::web::websocket::message::WsMessage;
    use actors::Signal;
    use serde::{Deserialize, Serialize};

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
        let ws_signal = WsMessage::new("SampleData", None, data.clone());

        let signal: Signal<SampleData> = ws_signal.into();

        assert_eq!(signal.data().lol, "lol");
        assert_eq!(signal.data().lel, "lel");

        assert!(signal.to().is_none());
    }
}
