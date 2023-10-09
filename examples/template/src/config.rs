pub mod http;
pub mod queue;
pub mod state;

use crate::cache::adapters::RedisAdapter as BasicCache;
use crate::core::auth::Authentication;
use crate::db::adapters::{session::SessionAdapter, user::UserAdapter};
use hextacy::adapters::{cache::redis::RedisDriver, db::sql::seaorm::SeaormDriver};

// Concretise services and hook them up to the framework specific configuration

pub type AuthenticationService =
    Authentication<SeaormDriver, RedisDriver, UserAdapter, SessionAdapter, BasicCache>;
