use crate::api::middleware::auth::adapter::{Cache as MwCache, Repo as MwRepo};
use crate::{api::router::auth::adapters::repository::Repo, db::models::role::Role};
use crate::{
    api::{
        middleware::auth::interceptor::AuthGuard,
        router::auth::{
            adapters::cache::Cache,
            o_auth::{handler, service::OAuthService},
        },
    },
    services::oauth::github::GithubOAuth,
};
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
        provider: GithubOAuth,
        repository: Repo::new(pg.clone()),
        cache: Cache { driver: rd.clone() },
    };

    let auth_guard = AuthGuard::<MwRepo, MwCache>::new(_pg, rd, Role::User);

    cfg.app_data(Data::new(service));

    cfg.service(
        web::resource("/auth/oauth/github/login")
            .route(web::post().to(handler::login::<OAuthService<GithubOAuth, Repo, Cache>>)),
    );

    cfg.service(
        web::resource("/auth/oauth/github/scope")
            .route(web::put().to(handler::request_scopes::<OAuthService<GithubOAuth, Repo, Cache>>))
            .wrap(auth_guard),
    );
}
