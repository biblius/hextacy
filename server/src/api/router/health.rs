use actix_web::{
    web::{self, Data, ServiceConfig},
    HttpResponseBuilder,
};
use infrastructure::clients::store::{postgres::Postgres, redis::Redis};
use reqwest::StatusCode;
use serde::Serialize;
use std::sync::Arc;

pub(crate) fn route(pg: Arc<Postgres>, rd: Arc<Redis>, cfg: &mut ServiceConfig) {
    let pools = Data::new(Pools { pg, rd });
    cfg.app_data(pools);
    cfg.service(web::resource("/health").route(web::get().to(health_check)));
}

async fn health_check(pools: web::Data<Pools>) -> impl actix_web::Responder {
    let pg_state = pools.pg.health_check();
    let rd_state = pools.rd.health_check();
    HttpResponseBuilder::new(StatusCode::OK).json(HealthCheck {
        message: "Ready to roll",
        pg_connections: pg_state.connections,
        pg_idle_connections: pg_state.idle_connections,
        rd_connections: rd_state.connections,
        rd_idle_connections: rd_state.idle_connections,
    })
}

struct Pools {
    pub pg: Arc<Postgres>,
    pub rd: Arc<Redis>,
}

#[derive(Debug, Serialize)]
struct HealthCheck {
    message: &'static str,
    pg_connections: u32,
    pg_idle_connections: u32,
    rd_connections: u32,
    rd_idle_connections: u32,
}
