use hextacy::RestResponse;
use serde::Serialize;

/// Holds a single message. Implements the Response trait as well as actix' Responder.
#[derive(Debug, Serialize, RestResponse)]
pub struct MessageResponse {
    message: String,
}

impl MessageResponse {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}
