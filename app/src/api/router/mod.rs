use actix_web::web::ServiceConfig;
use infrastructure::{
    email::lettre::SmtpTransport,
    storage::{postgres::Pg, redis::Rd},
};
use std::sync::Arc;

pub mod auth;

pub fn init(pg: Arc<Pg>, rd: Arc<Rd>, email_client: Arc<SmtpTransport>, cfg: &mut ServiceConfig) {
    auth::init(pg, rd, email_client, cfg);
}
