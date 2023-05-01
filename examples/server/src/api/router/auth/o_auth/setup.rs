use crate::api::middleware::auth::adapter::{Cache as MwCache, Repo as MwRepo};
use crate::api::{
    middleware::auth::interceptor::AuthenticationGuard,
    router::auth::{
        adapters::cache::Cache,
        o_auth::{handler, service::OAuthService},
    },
};
use crate::{api::router::auth::adapters::repository::Repo, db::models::role::Role};
use actix_web::web::{self, Data};
use hextacy::drivers::db::postgres::seaorm::PostgresSea;
use hextacy::drivers::{
    cache::redis::Redis,
    db::{mongo::Mongo, postgres::diesel::PostgresDiesel},
};
use std::sync::Arc;

pub(crate) fn routes(
    _pg: Arc<PostgresDiesel>,
    pg: Arc<PostgresSea>,
    rd: Arc<Redis>,
    _mg: Arc<Mongo>,
    cfg: &mut web::ServiceConfig,
) {
    let service = OAuthService {
        repository: Repo::new(pg),
        cache: Cache { driver: rd.clone() },
    };

    let auth_guard = AuthenticationGuard::<MwRepo, MwCache>::new(_pg, rd, Role::User);

    cfg.app_data(Data::new(service));

    cfg.service(
        web::resource("/auth/oauth/{provider}/login")
            .route(web::post().to(handler::login::<OAuthService<Repo, Cache>>)),
    );

    cfg.service(
        web::resource("/auth/oauth/{provider}/scope")
            .route(web::put().to(handler::request_scopes::<OAuthService<Repo, Cache>>))
            .wrap(auth_guard),
    );
}
