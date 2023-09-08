use crate::cache::adapters::redis::RedisAdapter;
use crate::db::adapters::postgres::diesel::user::PgUserAdapter as DieselUserAdapter;
use crate::db::adapters::postgres::seaorm::{
    oauth::PgOAuthAdapter, session::PgSessionAdapter, user::PgUserAdapter,
};
use crate::AppState;
use actix_web::web;
use hextacy::adapters::{
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

    pub type AuthenticationMiddleware = AuthenticationGuard<
        AuthMwRepo<PostgresSea, DatabaseConnection, PgSessionAdapter>,
        AuthMwCache<Redis, RedisConnection, RedisAdapter>,
    >;
}

pub(super) mod auth_service {
    use super::*;
    use crate::app::core::auth::{
        contracts::{
            cache::AuthenticationCacheAccess, email::Email,
            repository::AuthenticationRepositoryAccess,
        },
        native::Authentication,
    };

    pub type AuthenticationService = Authentication<
        AuthenticationRepositoryAccess<
            PostgresSea,
            DatabaseConnection,
            PgUserAdapter,
            PgSessionAdapter,
            PgOAuthAdapter,
        >,
        AuthenticationCacheAccess<Redis, RedisConnection, RedisAdapter>,
        Email,
    >;

    impl Configure<AppState> for AuthenticationService {
        fn configure(state: &AppState, cfg: &mut actix_web::web::ServiceConfig) {
            let service = Self {
                repository: AuthenticationRepositoryAccess::new(state.pg_sea.clone()),
                cache: AuthenticationCacheAccess::new(state.redis.clone()),
                email: Email::new(state.email.clone()),
            };
            cfg.app_data(web::Data::new(service));
        }
    }
}

pub(super) mod oauth_service {
    use super::*;
    use crate::app::core::auth::{
        contracts::{cache::AuthenticationCacheAccess, repository::AuthenticationRepositoryAccess},
        o_auth::OAuthService as Service,
    };

    pub type OAuthService = Service<
        AuthenticationRepositoryAccess<
            PostgresSea,
            DatabaseConnection,
            PgUserAdapter,
            PgSessionAdapter,
            PgOAuthAdapter,
        >,
        AuthenticationCacheAccess<Redis, RedisConnection, RedisAdapter>,
    >;

    impl Configure<AppState> for OAuthService {
        fn configure(state: &AppState, cfg: &mut actix_web::web::ServiceConfig) {
            let service = Self {
                repository: AuthenticationRepositoryAccess::new(state.pg_sea.clone()),
                cache: AuthenticationCacheAccess::new(state.redis.clone()),
            };
            cfg.app_data(web::Data::new(service));
        }
    }
}

pub(super) mod user_service {
    use super::*;
    use crate::app::core::users::{adapters::UsersRepository, domain::Users};

    pub type UserService =
        Users<UsersRepository<PostgresDiesel, DieselConnection, DieselUserAdapter>>;

    impl Configure<AppState> for UserService {
        fn configure(state: &AppState, cfg: &mut actix_web::web::ServiceConfig) {
            let service = Self {
                repository: UsersRepository::new(state.pg_diesel.clone()),
            };
            cfg.app_data(web::Data::new(service));
        }
    }
}
