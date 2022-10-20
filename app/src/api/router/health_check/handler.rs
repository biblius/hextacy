use actix_web::{web, HttpResponseBuilder};
use infrastructure::storage::{postgres::Pg, redis::Rd};
use reqwest::StatusCode;
use serde::Serialize;
use std::sync::Arc;

pub async fn health_check(pools: web::Data<Pools>) -> impl actix_web::Responder {
    let pg_state = pools.pg.health_check();
    let rd_state = pools.rd.health_check();
    HttpResponseBuilder::new(StatusCode::OK).json(HealthCheck {
        pg_connections: pg_state.connections,
        pg_idle_connections: pg_state.idle_connections,
        rd_connections: rd_state.connections,
        rd_idle_connections: rd_state.idle_connections,
    })
}

pub struct Pools {
    pub pg: Arc<Pg>,
    pub rd: Arc<Rd>,
}

#[derive(Debug, Serialize)]
struct HealthCheck {
    pg_connections: u32,
    pg_idle_connections: u32,
    rd_connections: u32,
    rd_idle_connections: u32,
}
