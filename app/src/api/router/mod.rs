mod auth;
mod health_check;
mod users;

use actix_web::web::{self, ServiceConfig};
use infrastructure::{
    email::lettre::SmtpTransport,
    storage::{postgres::Pg, redis::Rd},
};
use std::sync::Arc;

pub fn init(pg: Arc<Pg>, rd: Arc<Rd>, email_client: Arc<SmtpTransport>, cfg: &mut ServiceConfig) {
    // Configure health check
    cfg.service(
        web::scope("/health")
            .app_data(web::Data::new(health_check::handler::Pools {
                pg: pg.clone(),
                rd: rd.clone(),
            }))
            .service(
                web::resource("/pools").route(web::get().to(health_check::handler::health_check)),
            ),
    );

    auth::setup::init(pg.clone(), rd.clone(), email_client, cfg);

    users::setup::init(pg, rd, cfg);
}
