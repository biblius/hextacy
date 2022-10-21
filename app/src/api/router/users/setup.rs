use super::{handler, service::Users};
use crate::{api::middleware::auth::interceptor, models::role::Role};
use actix_web::web::{self, Data};
use infrastructure::storage::{postgres::Pg, redis::Rd};
use std::sync::Arc;

pub(crate) fn init(pg: Arc<Pg>, rd: Arc<Rd>, cfg: &mut web::ServiceConfig) {
    let service = Users::new(pg.clone());

    cfg.app_data(Data::new(service));

    // Show all
    cfg.service(
        web::resource("/users")
            .route(web::get().to(handler::get_paginated))
            .wrap(interceptor::Auth::new(pg, rd, Role::User)),
    );
}
