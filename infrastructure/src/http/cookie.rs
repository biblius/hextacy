use crate::config::constants::{COOKIE_EXPIRATION, COOKIE_SAME_SITE_DEFAULT};
use cookie::{Cookie, CookieBuilder, SameSite};
use serde::Serialize;

/// Creates a cookie with the given properties. SameSite defaults to Lax if None is provided.
pub fn create<'a, T: Serialize>(
    key: &'a str,
    value: &T,
    same_site: Option<SameSite>,
) -> Result<Cookie<'a>, serde_json::Error> {
    let json = serde_json::to_string(value)?;
    Ok(CookieBuilder::new(key, json)
        .max_age(COOKIE_EXPIRATION)
        .path("/")
        .same_site(same_site.unwrap_or(COOKIE_SAME_SITE_DEFAULT))
        .http_only(true)
        .secure(false)
        .finish())
}
