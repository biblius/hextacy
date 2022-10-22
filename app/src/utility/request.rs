use crate::error::{AuthenticationError, Error};
use actix_web::{HttpMessage, HttpRequest};
use infrastructure::repository::session::Session;

/// Utility for quickly dropping the request extensions reference and getting the
/// cloned session
#[inline]
pub fn extract_session(req: HttpRequest) -> Result<Session, Error> {
    let extensions = req.extensions();
    if let Some(ext) = extensions.get::<Session>() {
        Ok(ext.clone())
    } else {
        Err(Error::new(AuthenticationError::Unauthenticated))
    }
}
