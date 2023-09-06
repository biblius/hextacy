pub mod constants;
pub mod cors;

use crate::services::email::Email;
use hextacy::{
    adapters::{
        cache::redis::Redis,
        db::{
            mongo::Mongo,
            postgres::{diesel::PostgresDiesel, seaorm::PostgresSea},
        },
    },
    State,
};
use std::sync::Arc;

const REDIS_PORT: u16 = 6379;

#[derive(Debug, Clone, State)]
pub struct AppState {
    #[env(
        "PG_HOST",
        "PG_PORT" as u16,
        "PG_USER",
        "PG_PASSWORD",
        "PG_DATABASE",
        "PG_POOL_SIZE" as Option<u32>
    )]
    pub pg_diesel: PostgresDiesel,

    #[env(
        "PG_HOST",
        "PG_PORT" as u16,
        "PG_USER",
        "PG_PASSWORD",
        "PG_DATABASE",
        "PG_POOL_SIZE" as u32
    )]
    #[load_async]
    pub pg_sea: PostgresSea,

    #[env(
        "RD_HOST",
        "RD_PORT" as u16,
        "RD_USER" as Option,
        "RD_PASSWORD" as Option,
        "RD_DATABASE" as i64,
        "RD_POOL_SIZE" as usize
    )]
    #[raw("localhost", REDIS_PORT, None, None, 0, 8)]
    pub redis: Redis,

    #[env("SMTP_HOST", "SMTP_PORT" as u16, "SMTP_USERNAME", "SMTP_PASSWORD", "DOMAIN")]
    #[load_with(Email::new)]
    pub email: Arc<Email>,

    #[env(
        "MONGO_HOST",
        "MONGO_PORT" as u16,
        "MONGO_USER",
        "MONGO_PASSWORD",
        "MONGO_DATABASE"
    )]
    pub mongo: Mongo,
}
