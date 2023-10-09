use crate::{
    cache::adapters::RedisAdapter, config::state::AppState,
    controllers::http::middleware::auth::SessionGuard, db::adapters::session::SessionAdapter,
};
use hextacy::adapters::{cache::redis::RedisDriver, db::sql::seaorm::SeaormDriver};

pub type AuthenticationMiddleware =
    SessionGuard<SeaormDriver, RedisDriver, SessionAdapter, RedisAdapter>;

impl AuthenticationMiddleware {
    pub fn init(state: &AppState) -> AuthenticationMiddleware {
        AuthenticationMiddleware::new(
            state.repository.clone(),
            state.cache.clone(),
            SessionAdapter,
            RedisAdapter,
        )
    }
}
