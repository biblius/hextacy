use super::WsError;
use actix::Message;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Debug;
use tracing::error;

#[derive(Message)]
#[rtype(result = "()")]
pub struct RawJson(pub String);

impl RawJson {
    pub fn into_inner(self) -> String {
        self.0
    }

    pub fn get_inner(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsMessage<T> {
    pub domain: String,
    pub data: T,
}

impl<T> WsMessage<T>
where
    T: Serialize + DeserializeOwned + Debug,
{
    pub fn new(domain: String, data: T) -> Self {
        Self { domain, data }
    }

    pub fn to_json(&self) -> Result<RawJson, WsError> {
        let s = serde_json::to_string(&self).map_err(|e| {
            error!("An error occurred while serializing to JSON: {}", e);
            WsError::Serde(e)
        })?;
        Ok(RawJson(s))
    }

    pub fn from_json(json: &str) -> Result<WsMessage<T>, WsError> {
        let s = serde_json::from_str::<WsMessage<T>>(json).map_err(|e| {
            error!("An error occurred while deserializing to WsMessage: {}", e);
            WsError::Serde(e)
        })?;
        Ok(s)
    }

    /// Testing purposes. Generates a `WsMessage` with `domain: "mock"` and the given data
    pub fn __mock(data: T) -> Self {
        Self {
            domain: "mock".to_string(),
            data,
        }
    }
}
