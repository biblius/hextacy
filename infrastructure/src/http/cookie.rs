use crate::config::constants::{COOKIE_EXPIRATION, COOKIE_SAME_SITE_DEFAULT};
use cookie::{Cookie, CookieBuilder, SameSite};
use serde::Serialize;

/// Creates an HTTP only cookie
pub fn csrf<'a>(token: &'a str) -> Cookie<'a> {
    CookieBuilder::new("x-csrf-token", token)
        .max_age(COOKIE_EXPIRATION)
        .same_site(cookie::SameSite::None)
        .http_only(true)
        .secure(true)
        .finish()
}

/// Creates a cookie with the given params
pub fn create<'a, T: Serialize>(
    key: &'a str,
    value: &T,
    same_site: Option<SameSite>,
) -> Result<Cookie<'a>, serde_json::Error> {
    let json = serde_json::to_string(value)?;
    Ok(CookieBuilder::new(key, json)
        .max_age(COOKIE_EXPIRATION)
        .same_site(same_site.unwrap_or_else(|| COOKIE_SAME_SITE_DEFAULT))
        .http_only(false)
        .secure(true)
        .finish())
}
