pub mod auth;
pub mod data;
pub mod middleware;
pub mod resources;

use axum_extra::extract::cookie::{Cookie, SameSite};
use hextacy::web::{cookie::time::Duration, cookie::CookieBuilder};

const PATH: &str = "/";
const HTTP_ONLY: bool = true;
const SECURE: bool = true;
const DOMAIN: &str = "";
const MAX_AGE: Duration = Duration::days(1);
const SAME_SITE: SameSite = SameSite::Lax;

/// Creates a cookie with the given properties.
pub fn create_session_cookie<'a>(
    key: &'a str,
    value: &'a str,
    expire: bool,
) -> Result<Cookie<'a>, serde_json::Error> {
    Ok(CookieBuilder::new(key, value)
        .path(PATH)
        .domain(DOMAIN)
        .max_age(if expire { Duration::ZERO } else { MAX_AGE })
        .same_site(SAME_SITE)
        .http_only(HTTP_ONLY)
        .secure(SECURE)
        .finish())
}
