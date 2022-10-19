pub mod auth;
pub mod users;

use actix_web::web::ServiceConfig;
use infrastructure::{
    email::lettre::SmtpTransport,
    storage::{postgres::Pg, redis::Rd},
};
use std::sync::Arc;

pub fn init(pg: Arc<Pg>, rd: Arc<Rd>, email_client: Arc<SmtpTransport>, cfg: &mut ServiceConfig) {
    auth::setup::init(pg.clone(), rd.clone(), email_client, cfg);
    users::setup::init(pg, rd, cfg);
}
