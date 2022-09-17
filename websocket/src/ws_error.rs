use std::fmt::Display;

#[derive(Debug)]
pub enum WsError {
    Serde(serde_json::error::Error),
}

impl Display for WsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Serde(e) => writeln!(f, "Serde error occurred: {e}"),
        }
    }
}

impl From<serde_json::error::Error> for WsError {
    fn from(error: serde_json::error::Error) -> WsError {
        WsError::Serde(error)
    }
}
