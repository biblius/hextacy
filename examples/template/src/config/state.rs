use crate::cache::adapters::RedisAdapter as BasicCache;
use crate::cache::driver::RedisDriver;
use crate::core::auth::Authentication;
use crate::db::adapters::session::SessionAdapter;
use crate::db::adapters::user::UserAdapter;
use crate::db::driver::SeaormDriver;
use crate::{cache::adapters::RedisAdapter, controllers::http::middleware::auth::SessionGuard};
use hextacy::adapters::queue::redis::RedisMessageQueue;
use hextacy::adapters::queue::redis::RedisPublisher;
use hextacy::State;

#[derive(Debug, Clone, State)]
pub struct AppState {
    #[env("DATABASE_URL")]
    #[load_async]
    pub repository: SeaormDriver,

    #[env(
        "RD_HOST",
        "RD_PORT" as u16,
        "RD_USER" as Option,
        "RD_PASSWORD" as Option,
        "RD_DATABASE" as i64,
    )]
    pub cache: RedisDriver,

    #[env(
        "RD_HOST",
        "RD_PORT" as u16,
        "RD_USER" as Option,
        "RD_PASSWORD" as Option,
    )]
    pub redis_q: RedisMessageQueue,
}

// Concretise services

pub type AuthenticationService = Authentication<
    SeaormDriver,
    RedisDriver,
    UserAdapter,
    SessionAdapter,
    BasicCache,
    RedisPublisher,
>;

impl AuthenticationService {
    pub async fn init(state: &AppState) -> AuthenticationService {
        AuthenticationService::new(
            state.repository.clone(),
            state.cache.clone(),
            UserAdapter,
            SessionAdapter,
            RedisAdapter,
            state
                .redis_q
                .publisher("my-channel")
                .await
                .expect("Could not create publisher"),
        )
    }
}

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
