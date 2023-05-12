use crate::api::middleware::auth::adapter::{Cache as MwCache, Repo as MwRepo};
use crate::api::router::auth::adapters::repository::RepositoryComponent;
use crate::api::{
    middleware::auth::interceptor::AuthenticationGuard,
    router::auth::{
        adapters::cache::AuthenticationCache,
        o_auth::{handler, service::OAuthService},
    },
};
use crate::cache::adapters::redis::RedisAdapter;
use crate::config::AppState;
use crate::db::adapters::postgres::seaorm::oauth::PgOAuthAdapter;
use crate::db::adapters::postgres::seaorm::session::PgSessionAdapter;
use crate::db::adapters::postgres::seaorm::user::PgUserAdapter;
use crate::db::models::role::Role;
use actix_web::web::{self, Data};
use hextacy::drivers::cache::redis::{Redis, RedisConnection};
use hextacy::drivers::db::postgres::seaorm::PostgresSea;
use sea_orm::DatabaseConnection;

type CacheComponent = AuthenticationCache<Redis, RedisConnection, RedisAdapter>;

type RepoComponent = RepositoryComponent<
    PostgresSea,
    DatabaseConnection,
    PgUserAdapter,
    PgSessionAdapter,
    PgOAuthAdapter,
>;

pub(crate) fn routes(AppState { pg_sea, redis, .. }: &AppState, cfg: &mut web::ServiceConfig) {
    let service = OAuthService {
        repository: RepoComponent::new(pg_sea.clone()),
        cache: CacheComponent::new(redis.clone()),
    };

    let auth_guard =
        AuthenticationGuard::<MwRepo, MwCache>::new(pg_sea.clone(), redis.clone(), Role::User);

    cfg.app_data(Data::new(service));

    cfg.service(
        web::resource("/auth/oauth/{provider}/login")
            .route(web::post().to(handler::login::<OAuthService<RepoComponent, CacheComponent>>)),
    );

    cfg.service(
        web::resource("/auth/oauth/{provider}/scope")
            .route(
                web::put()
                    .to(handler::request_scopes::<OAuthService<RepoComponent, CacheComponent>>),
            )
            .wrap(auth_guard),
    );
}
