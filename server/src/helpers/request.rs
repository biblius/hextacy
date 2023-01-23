use actix_web::{HttpMessage, HttpRequest};
use infrastructure::web::http::HttpError;
use storage::models::session::UserSession;

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