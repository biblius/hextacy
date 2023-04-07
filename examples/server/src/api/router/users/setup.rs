use super::adapter::Repository;
use super::{handler, service::UserService};
use crate::db::adapters::postgres::seaorm::user::PgUserAdapter;
use actix_web::web::{self, Data};
use hextacy::drivers::cache::redis::Redis;
use hextacy::drivers::db::postgres::seaorm::PostgresSea;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

pub(crate) fn routes(pg: Arc<PostgresSea>, _rd: Arc<Redis>, cfg: &mut web::ServiceConfig) {
    let service = UserService {
        repository: Repository::<PostgresSea, DatabaseConnection, PgUserAdapter>::new(pg.clone()),
    };

    cfg.app_data(Data::new(service));

    // Show all
    cfg.service(
        web::resource("/users").route(web::get().to(handler::get_paginated::<
            UserService<Repository<PostgresSea, DatabaseConnection, PgUserAdapter>>,
        >)),
    );
}
