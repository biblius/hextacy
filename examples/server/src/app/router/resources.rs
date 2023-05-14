use crate::helpers::resources::FAVICON;
use actix_web::web::{self, ServiceConfig};
use actix_web::{body::BoxBody, HttpResponseBuilder, Responder};
use reqwest::{header, StatusCode};

async fn favicon() -> impl Responder {
    HttpResponseBuilder::new(StatusCode::OK)
        .append_header((
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("image/x-ico"),
        ))
        .body(BoxBody::new(FAVICON.clone()))
}

pub(crate) fn route(cfg: &mut ServiceConfig) {
    cfg.service(web::resource("/favicon.ico").route(web::get().to(favicon)));
}
