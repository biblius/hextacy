use crate::helpers::lazy::resources::FAVICON;
use actix_web::{body::BoxBody, HttpResponseBuilder, Responder};
use reqwest::{header, StatusCode};

pub(super) async fn favicon() -> impl Responder {
    HttpResponseBuilder::new(StatusCode::OK)
        .append_header((
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("image/x-ico"),
        ))
        .body(BoxBody::new(FAVICON.clone()))
}
