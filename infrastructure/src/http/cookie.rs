use cookie::{time::Duration, Cookie, CookieBuilder, SameSite};
use serde::Serialize;

// Cookie identifiers
pub const S_ID: &str = "S_ID";

const PATH: &str = "/";
const HTTP_ONLY: bool = true;
const SECURE: bool = false;
const DOMAIN: &str = "";
const MAX_AGE: Duration = Duration::days(1);
const SAME_SITE: SameSite = SameSite::Lax;

/// Creates a cookie with the given session id
pub fn create_session(session_id: &str, expire: bool, permanent: bool) -> Cookie<'_> {
    CookieBuilder::new(S_ID, session_id)
        .path(PATH)
        .domain(DOMAIN)
        .max_age(if expire {
            Duration::ZERO
        } else if permanent {
            Duration::MAX
        } else {
            MAX_AGE
        })
        .same_site(SAME_SITE)
        .http_only(HTTP_ONLY)
        .secure(SECURE)
        .finish()
}

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
