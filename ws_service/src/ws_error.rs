use std::fmt::Display;

use mycro_core::signal::SignalError;

#[derive(Debug)]
pub enum WsError {
    Serde(serde_json::error::Error),
    Signal(SignalError),
}

impl Display for WsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Serde(e) => writeln!(f, "Serde error occurred: {e}"),
            Self::Signal(e) => writeln!(f, "Signal error occurred: {:?}", e),
        }
    }
}

impl From<serde_json::error::Error> for WsError {
    fn from(error: serde_json::error::Error) -> WsError {
        WsError::Serde(error)
    }
}
impl From<SignalError> for WsError {
    fn from(error: SignalError) -> WsError {
        WsError::Signal(error)
    }
}
