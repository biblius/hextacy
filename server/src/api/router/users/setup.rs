use super::{domain::UserService, handler, infrastructure::Repository};
use crate::api::middleware::auth::interceptor;
use actix_web::web::{self, Data};
use infrastructure::{
    clients::store::{postgres::Postgres, redis::Redis},
    store::adapters::postgres::user::PgUserAdapter,
    store::repository::role::Role,
};
use std::sync::Arc;

pub(crate) fn routes(pg: Arc<Postgres>, rd: Arc<Redis>, cfg: &mut web::ServiceConfig) {
    let service = UserService {
        repository: Repository {
            user_repo: PgUserAdapter { client: pg.clone() },
        },
    };

    let guard = interceptor::AuthGuard::new(pg, rd, Role::User);

    cfg.app_data(Data::new(service));

    // Show all
    cfg.service(
        web::resource("/users")
            .route(web::get().to(handler::get_paginated::<UserService<Repository<PgUserAdapter>>>))
            .wrap(guard),
    );
}
