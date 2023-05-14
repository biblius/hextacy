use actix_web::{
    web::{self, ServiceConfig},
    HttpResponseBuilder,
};
use reqwest::StatusCode;
use serde::Serialize;

pub(crate) fn route(cfg: &mut ServiceConfig) {
    cfg.service(web::resource("/health").route(web::get().to(health_check)));
}

async fn health_check() -> impl actix_web::Responder {
    HttpResponseBuilder::new(StatusCode::OK).json(HealthCheck {
        message: "Ready to roll",
    })
}

#[derive(Debug, Serialize)]
struct HealthCheck {
    message: &'static str,
}
