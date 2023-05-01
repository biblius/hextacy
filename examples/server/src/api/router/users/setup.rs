use super::adapter::Repository;
use super::{handler, service::UserService};
use crate::api::middleware::auth::{
    adapter::{Cache as MwCache, Repo as MwRepo},
    interceptor,
};
use crate::db::adapters::postgres::diesel::user::PgUserAdapter;
use crate::db::models::role::Role;
use actix_web::web::{self, Data};
use hextacy::drivers::cache::redis::Redis;
use hextacy::drivers::db::postgres::diesel::{PgPoolConnection, PostgresDiesel};
use std::sync::Arc;

pub(crate) fn routes(pg: Arc<PostgresDiesel>, rd: Arc<Redis>, cfg: &mut web::ServiceConfig) {
    let service = UserService {
        repository: Repository::<PostgresDiesel, PgPoolConnection, PgUserAdapter>::new(pg.clone()),
    };

    let session_guard =
        interceptor::AuthenticationGuard::<MwRepo, MwCache>::new(pg, rd, Role::User);

    cfg.app_data(Data::new(service));

    // Show all
    cfg.service(
        web::resource("/users")
            .route(web::get().to(handler::Handler::get_paginated::<
                UserService<Repository<PostgresDiesel, PgPoolConnection, PgUserAdapter>>,
            >))
            .wrap(session_guard),
    );
}
