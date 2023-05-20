use crate::cache::adapters::redis::RedisAdapter;
use crate::db::adapters::postgres::diesel::user::PgUserAdapter as DieselUserAdapter;
use crate::db::adapters::postgres::seaorm::{
    oauth::PgOAuthAdapter, session::PgSessionAdapter, user::PgUserAdapter,
};
use crate::AppState;
use actix_web::web;
use hextacy::drivers::{
    cache::redis::{Redis, RedisConnection},
    db::postgres::{
        diesel::{DieselConnection, PostgresDiesel},
        seaorm::PostgresSea,
    },
};
use hextacy::web::Configure;
use sea_orm::DatabaseConnection;

pub(super) mod auth_middleware {
    use super::*;
    use crate::app::middleware::auth::{
        adapter::{AuthMwCache, AuthMwRepo},
        interceptor::AuthenticationGuard,
    };

    type AuthMWCache = AuthMwCache<Redis, RedisConnection, RedisAdapter>;
    type AuthMWRepo = AuthMwRepo<PostgresSea, DatabaseConnection, PgSessionAdapter>;
    pub type AuthenticationMiddleware = AuthenticationGuard<AuthMWRepo, AuthMWCache>;
}

pub(super) mod auth_service {
    use super::*;
    use crate::app::core::auth::{
        adapters::{
            cache::AuthenticationCache, email::Email, repository::AuthenticationRepository,
        },
        native::Authentication,
    };

    pub type CacheComponent = AuthenticationCache<Redis, RedisConnection, RedisAdapter>;
    pub type RepoComponent = AuthenticationRepository<
        PostgresSea,
        DatabaseConnection,
        PgUserAdapter,
        PgSessionAdapter,
        PgOAuthAdapter,
    >;
    pub type EmailComponent = Email;

    pub type AuthenticationService = Authentication<RepoComponent, CacheComponent, Email>;

    impl Configure<AppState> for AuthenticationService {
        fn configure(state: &AppState, cfg: &mut actix_web::web::ServiceConfig) {
            let service = Self {
                repository: RepoComponent::new(state.pg_sea.clone()),
                cache: CacheComponent::new(state.redis.clone()),
                email: EmailComponent {
                    driver: state.smtp.clone(),
                },
            };
            cfg.app_data(web::Data::new(service));
        }
    }
}

pub(super) mod oauth_service {
    use super::*;
    use crate::app::core::auth::{
        adapters::{cache::AuthenticationCache, repository::AuthenticationRepository},
        o_auth::OAuthService as Service,
    };

    pub type CacheComponent = AuthenticationCache<Redis, RedisConnection, RedisAdapter>;
    pub type RepoComponent = AuthenticationRepository<
        PostgresSea,
        DatabaseConnection,
        PgUserAdapter,
        PgSessionAdapter,
        PgOAuthAdapter,
    >;

    pub type OAuthService = Service<RepoComponent, CacheComponent>;

    impl Configure<AppState> for OAuthService {
        fn configure(state: &AppState, cfg: &mut actix_web::web::ServiceConfig) {
            let service = Self {
                repository: RepoComponent::new(state.pg_sea.clone()),
                cache: CacheComponent::new(state.redis.clone()),
            };
            cfg.app_data(web::Data::new(service));
        }
    }
}

pub(super) mod user_service {
    use super::*;
    use crate::app::core::users::{adapters::Repository, service::UserService as Service};

    pub type RepoComponent = Repository<PostgresDiesel, DieselConnection, DieselUserAdapter>;

    pub type UserService = Service<RepoComponent>;

    impl Configure<AppState> for UserService {
        fn configure(state: &AppState, cfg: &mut actix_web::web::ServiceConfig) {
            let service = Self {
                repository: RepoComponent::new(state.pg_diesel.clone()),
            };
            cfg.app_data(web::Data::new(service));
        }
    }
}
