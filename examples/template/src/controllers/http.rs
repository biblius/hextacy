pub mod auth;
pub mod middleware;
pub mod resources;

use axum_extra::extract::cookie::{Cookie, SameSite};
use hextacy::web::{cookie::time::Duration, cookie::CookieBuilder};
use hextacy::RestResponse;
use serde::Serialize;

const PATH: &str = "/";
const HTTP_ONLY: bool = true;
const SECURE: bool = true;
const DOMAIN: &str = "";
const MAX_AGE: Duration = Duration::days(1);
const SAME_SITE: SameSite = SameSite::Lax;

pub fn session_cookie<'a>(key: &'a str, value: &'a str, expire: bool) -> Cookie<'a> {
    CookieBuilder::new(key, value)
        .path(PATH)
        .domain(DOMAIN)
        .max_age(if expire { Duration::ZERO } else { MAX_AGE })
        .same_site(SAME_SITE)
        .http_only(HTTP_ONLY)
        .secure(SECURE)
        .finish()
}

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
