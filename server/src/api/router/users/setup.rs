use super::{domain::UserService, handler};
use crate::api::middleware::auth::interceptor;
use actix_web::web::{self, Data};
use infrastructure::{
    clients::storage::{postgres::Postgres, redis::Redis},
    storage::adapters::postgres::user::PgUserAdapter,
    storage::repository::role::Role,
};
use std::sync::Arc;

pub(crate) fn routes(pg: Arc<Postgres>, rd: Arc<Redis>, cfg: &mut web::ServiceConfig) {
    let service = UserService {
        repository: PgUserAdapter { client: pg.clone() },
    };
    let auth_guard = interceptor::AuthGuard::new(pg, rd, Role::User);

    cfg.app_data(Data::new(service));

    // Show all
    cfg.service(
        web::resource("/users")
            .route(web::get().to(handler::get_paginated::<UserService<PgUserAdapter>>))
            .wrap(auth_guard),
    );
}
