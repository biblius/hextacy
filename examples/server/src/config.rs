pub mod constants;
pub mod cors;

use self::constants::EMAIL_DIRECTORY;
use hextacy::{
    adapters::{
        cache::redis::RedisDriver,
        db::{
            mongo::MongoDriver,
            sql::{diesel::DieselDriver, seaorm::SeaormDriver},
        },
        email::SimpleTemplateMailer,
    },
    State,
};
use std::sync::Arc;

#[derive(Debug, Clone, State)]
pub struct AppState {
    #[env("POSTGRES_URL")]
    pub pg_diesel: DieselDriver,

    #[env("POSTGRES_URL")]
    #[load_async]
    pub pg_sea: SeaormDriver,

    #[env(
        "RD_HOST",
        "RD_PORT" as u16,
        "RD_USER" as Option,
        "RD_PASSWORD" as Option,
        "RD_DATABASE" as i64,
    )]
    #[raw("127.0.0.1", 6379, None, None, 0)]
    pub redis: RedisDriver,

    #[env("SMTP_HOST", "SMTP_PORT" as u16, "SMTP_USERNAME", "SMTP_PASSWORD", "SMTP_FROM", "SMTP_SENDER")]
    pub email: std::sync::Arc<SimpleTemplateMailer>,

    #[env(
        "MONGO_HOST",
        "MONGO_PORT" as u16,
        "MONGO_USER",
        "MONGO_PASSWORD",
        "MONGO_DATABASE"
    )]
    pub mongo: MongoDriver,
}

impl AppState {
    pub async fn init() -> Result<Self, AppStateConfigurationError> {
        let mut email = Self::load_email_env()?;
        email
            .load_templates(EMAIL_DIRECTORY)
            .expect("Could not load email templates");
        Ok(Self {
            pg_diesel: Self::load_pg_diesel_env()?,
            pg_sea: Self::load_pg_sea_env().await?,
            redis: Self::load_redis_env()?,
            mongo: Self::load_mongo_env()?,
            email: Arc::new(email),
        })
    }
}
