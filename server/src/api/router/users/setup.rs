use super::{domain::UserService, handler};
use crate::api::middleware::auth::interceptor;
use actix_web::web::{self, Data};
use infrastructure::clients::{postgres::Postgres, redis::Redis};
use std::sync::Arc;
use storage::adapters::postgres::user::PgUserAdapter;
use storage::models::role::Role;

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
