use actix_web::{HttpMessage, HttpRequest};

use crate::store::models::user_session::UserSession;

use super::HttpError;

/// Utility for quickly dropping the request extensions reference and getting the
/// cloned session
#[inline]
pub fn extract_session(req: HttpRequest) -> Result<UserSession, HttpError> {
    let extensions = req.extensions();
    if let Some(ext) = extensions.get::<UserSession>() {
        Ok(ext.clone())
    } else {
        Err(HttpError::Request(String::from(
            "Could not extract session",
        )))
    }
}
