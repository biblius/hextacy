use crate::db::models::session::Session;
use actix_web::{HttpMessage, HttpRequest};
use hextacy::web::http::HttpError;

/// Utility for quickly dropping the request extensions reference and getting the
/// cloned session
pub fn extract_session(req: HttpRequest) -> Result<Session, HttpError> {
    let extensions = req.extensions();
    if let Some(ext) = extensions.get::<Session>() {
        Ok(ext.clone())
    } else {
        Err(HttpError::Request(String::from(
            "Could not extract session",
        )))
    }
}
