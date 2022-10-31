use crate::helpers::lazy::resources::FAVICON;
use actix_web::{
    body::BoxBody,
    web::{self, ServiceConfig},
    HttpResponseBuilder,
};
use reqwest::{header, StatusCode};

pub(crate) fn route(cfg: &mut ServiceConfig) {
    cfg.service(web::resource("/favicon.ico").route(web::get().to(favicon)));
}

async fn favicon() -> impl actix_web::Responder {
    HttpResponseBuilder::new(StatusCode::OK)
        .append_header((
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("image/x-ico"),
        ))
        .body(BoxBody::new(FAVICON.clone()))
}
