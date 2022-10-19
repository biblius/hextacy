use crate::api::middleware::auth::interceptor;

use super::{handler, service::Users};
use actix_web::web::{self, Data};
use infrastructure::storage::{postgres::Pg, redis::Rd};
use std::sync::Arc;

pub fn init(pg: Arc<Pg>, rd: Arc<Rd>, cfg: &mut web::ServiceConfig) {
    let service = Users::new(pg.clone());

    cfg.app_data(Data::new(service));

    // Show all
    cfg.service(
        web::resource("/users/show-all")
            .route(web::get().to(handler::get_all))
            .wrap(interceptor::Auth::new(pg, rd)),
    );
}
