use crate::{
    db::models::session::Session,
    error::{AuthenticationError, Error},
};
use actix_web::{HttpMessage, HttpRequest};

/// Utility for quickly dropping the request extensions reference and getting the
/// cloned session
pub fn extract_session(req: HttpRequest) -> Result<Session, Error> {
    let extensions = req.extensions();
    if let Some(ext) = extensions.get::<Session>() {
        Ok(ext.clone())
    } else {
        Err(AuthenticationError::Unauthenticated.into())
    }
}
