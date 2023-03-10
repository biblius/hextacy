use super::adapter::Repository;
use super::{handler, service::UserService};
use crate::api::middleware::auth::interceptor;
use crate::db::adapters::postgres::user::PgUserAdapter;
use crate::db::models::role::Role;
use actix_web::web::{self, Data};
use hextacy::clients::db::postgres::PgPoolConnection;
use hextacy::clients::db::{postgres::Postgres, redis::Redis};
use std::sync::Arc;

pub(crate) fn routes(pg: Arc<Postgres>, rd: Arc<Redis>, cfg: &mut web::ServiceConfig) {
    let service = UserService {
        repository: Repository::<Postgres, PgPoolConnection, PgUserAdapter>::new(pg.clone()),
    };
    let auth_guard = interceptor::AuthGuard::new(pg, rd, Role::User);

    cfg.app_data(Data::new(service));

    // Show all
    cfg.service(
        web::resource("/users")
            .route(web::get().to(handler::get_paginated::<
                UserService<Repository<Postgres, PgPoolConnection, PgUserAdapter>>,
            >))
            .wrap(auth_guard),
    );
}
