use actix_web::cookie::{time::Duration, Cookie, CookieBuilder, SameSite};
use serde::Serialize;

const PATH: &str = "/";
const HTTP_ONLY: bool = true;
const SECURE: bool = true;
const DOMAIN: &str = "";
const MAX_AGE: Duration = Duration::days(1);
const SAME_SITE: SameSite = SameSite::Lax;

/// Creates a cookie with the given properties.
pub fn create<'a, T: Serialize>(
    key: &'a str,
    value: &T,
    expire: bool,
) -> Result<Cookie<'a>, serde_json::Error> {
    let json = serde_json::to_string(value)?;
    Ok(CookieBuilder::new(key, json)
        .path(PATH)
        .domain(DOMAIN)
        .max_age(if expire { Duration::ZERO } else { MAX_AGE })
        .same_site(SAME_SITE)
        .http_only(HTTP_ONLY)
        .secure(SECURE)
        .finish())
}
