use super::adapter::Repository;
use super::{handler, service::UserService};
use crate::db::adapters::postgres::diesel::user::PgUserAdapter;
use actix_web::web::{self, Data};
use hextacy::drivers::cache::redis::Redis;
use hextacy::drivers::db::postgres::diesel::{DieselConnection, PostgresDiesel};
use std::sync::Arc;

pub(crate) fn routes(pg: Arc<PostgresDiesel>, _rd: Arc<Redis>, cfg: &mut web::ServiceConfig) {
    let service = UserService {
        repository: Repository::<PostgresDiesel, DieselConnection, PgUserAdapter>::new(pg),
    };

    /*     let session_guard =
    interceptor::AuthenticationGuard::<MwRepo, MwCache>::new(pg, rd, Role::User); */

    cfg.app_data(Data::new(service));

    // Show all
    cfg.service(
        web::resource("/users").route(web::get().to(handler::Handler::get_paginated::<
            UserService<Repository<PostgresDiesel, DieselConnection, PgUserAdapter>>,
        >)),
    );
}
