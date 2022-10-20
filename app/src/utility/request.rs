use crate::{
    error::{AuthenticationError, Error},
    models::session::Session,
};
use actix_web::{HttpMessage, HttpRequest};

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
