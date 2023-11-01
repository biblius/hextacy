use crate::cache::contracts::BasicCacheAccess;
use crate::config::state::AuthenticationMiddleware;
use crate::core::auth::AuthenticationError;
use crate::core::models;
use crate::core::repository::session::SessionRepository;
use crate::error::Error;
use crate::AppResult;
use axum::extract::State;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
use axum_extra::extract::CookieJar;
use hextacy::component;
use uuid::Uuid;

#[component(
    use Repo as repo,
    use Cache as cache,

    use SessionRepo, Cacher
)]
#[derive(Debug, Clone)]
pub struct SessionGuard {}

#[component(
    use Repo for Session: SessionRepository,
    use Cache for Cacher: BasicCacheAccess,
)]
impl SessionGuard {
    pub async fn get_session(
        &self,
        id: Uuid,
        csrf: Uuid,
    ) -> AppResult<Option<models::session::Session>> {
        let mut conn = self.repo.connect().await?;
        self.session_repo
            .get_valid_by_id(&mut conn, id, csrf)
            .await
            .map_err(Error::new)
    }
}

pub async fn session_check<B>(
    State(guard): State<AuthenticationMiddleware>,
    cookies: CookieJar,
    mut req: Request<B>,
    next: Next<B>,
) -> AppResult<Response> {
    let csrf = req
        .headers()
        .get("x-csrf-token")
        .ok_or(Error::new(AuthenticationError::Unauthenticated))?
        .to_str()
        .map_err(|_| AuthenticationError::Unauthenticated)?;

    dbg!(csrf);

    let id = cookies
        .get("S_ID")
        .ok_or(Error::new(AuthenticationError::Unauthenticated))?
        .value();

    dbg!(id, csrf);

    let csrf = Uuid::parse_str(csrf).map_err(Error::new)?;

    let id = Uuid::parse_str(id).map_err(Error::new)?;

    let Some(session) = guard.get_session(id, csrf).await? else {
        return Err(Error::new(AuthenticationError::Unauthenticated));
    };

    req.extensions_mut().insert(session);

    let response = next.run(req).await;

    Ok(response)
}

/* pub(super) fn security_headers() -> DefaultHeaders {
    use hextacy::web::xhttp::security_headers::*;
    DefaultHeaders::new()
        .add(default_content_security_policy())
        .add(cross_origin_embedder_policy("require-corp"))
        .add(cross_origin_opener_policy("same-origin"))
        .add(cross_origin_resource_policy("same-origin"))
        .add(referrer_policy(&["no-referrer", "same-origin"]))
        .add(strict_transport_security(
            31536000, // 1 year
            Some("includeSubDomains"),
        ))
        .add(no_sniff())
        .add(dns_prefetch_control(false))
        .add(ie_no_open())
        .add(frame_options(true))
        .add(cross_domain_policies("none"))
        .add(xss_filter(false))
}
 */

#[cfg(test)]
mod tests {}
