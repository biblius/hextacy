use super::favicon::favicon;
use actix_web::web::{self, ServiceConfig};

pub fn routes(cfg: &mut ServiceConfig) {
    cfg.service(web::resource("/favicon.ico").route(web::get().to(favicon)));
}
