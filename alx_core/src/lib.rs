pub mod cache;
pub mod crypto;
pub mod web;

pub use cache::CacheAccess;
pub use clients;
pub use utils::env;
pub use web::ws::broker;

pub mod time {
    pub fn now() -> i64 {
        chrono::Utc::now().timestamp()
    }
    pub fn date_now() -> chrono::DateTime<chrono::Utc> {
        chrono::Utc::now()
    }
}
