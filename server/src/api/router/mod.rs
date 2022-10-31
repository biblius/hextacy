pub(crate) mod auth;
mod health_check;
pub(crate) mod users;

use actix_web::web::{self, ServiceConfig};
use infrastructure::clients::{
    email::lettre::SmtpTransport,
    store::{postgres::Postgres, redis::Redis},
};
use std::sync::Arc;

pub fn init(
    pg: Arc<Postgres>,
    rd: Arc<Redis>,
    email_client: Arc<SmtpTransport>,
    cfg: &mut ServiceConfig,
) {
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

    auth::setup::routes(pg.clone(), rd.clone(), email_client, cfg);

    users::setup::routes(pg, rd, cfg);
}
