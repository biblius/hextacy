use super::WsError;
use actix::{Message, Recipient};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Debug;
use tracing::error;

#[derive(Message)]
#[rtype(result = "()")]
/// Message containing a JSON string to be sent to the ws session handler.
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
/// Message obtained via the ws session, deserialized from JSON. The domain is used
/// to broadcast the message to all actors subscribed to the domain's data type.
pub struct WsMessage<T> {
    pub domain: String,
    pub data: T,
}

impl<T> WsMessage<T>
where
    T: DeserializeOwned + Serialize,
{
    pub fn to_json(&self) -> Result<RawJson, WsError> {
        let s = serde_json::to_string(&self).map_err(|e| {
            error!("An error occurred while serializing to JSON: {}", e);
            WsError::Parser(e)
        })?;
        Ok(RawJson(s))
    }

    pub fn to_actor_message(id: &str, json: &str) -> Result<ActorMessage<T>, WsError> {
        let data = serde_json::from_str::<WsMessage<T>>(json.trim())?.data;
        Ok(ActorMessage {
            sender_id: id.to_string(),
            data,
        })
    }

    /// Testing purposes. Generates a `WsMessage` with `domain: "mock"` and the given data
    #[doc(hidden)]
    #[cfg(test)]
    pub fn __mock(data: T) -> Self {
        Self {
            domain: "mock".to_string(),
            data,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Message)]
#[rtype("()")]
pub struct ActorMessage<T> {
    sender_id: String,
    data: T,
}

impl<T> ActorMessage<T> {
    pub fn new(sender_id: String, data: T) -> Self {
        Self { sender_id, data }
    }

    pub fn sender(&self) -> &str {
        &self.sender_id
    }

    pub fn into_inner(self) -> T {
        self.data
    }
}

/// Used to inform other actors that an actor has started. Actors can use this message to
/// add the session actor to their address book.
#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct Connect {
    /// The ID of the session to connect
    pub session_id: String,

    /// The address of the session actor to send the messages to
    pub address: Recipient<RawJson>,
}

/// Removes the corresponding actor from the actor's address book
#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct Disconnect {
    /// The ID of the session to disconnect
    pub session_id: String,
}
