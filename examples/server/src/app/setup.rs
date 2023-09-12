use crate::AppState;
use actix_web::web;
use hextacy::web::Configure;

// Database
use hextacy::adapters::{
    cache::redis::RedisDriver, db::postgres::diesel::DieselPgDriver as Database,
};

// Cache
use crate::cache::adapters::redis::RedisAdapter as BasicCache;

// Adapters
use crate::db::adapters::postgres::diesel::{
    oauth::PgOAuthAdapter as OAuthAdapter, session::PgSessionAdapter as SessionAdapter,
    user::PgUserAdapter as UserAdapter,
};

/* use crate::db::adapters::postgres::seaorm::{
    oauth::PgOAuthAdapter, session::PgSessionAdapter, user::PgUserAdapter,
}; */

pub(super) mod auth_middleware {
    use super::*;
    use crate::{
        app::middleware::auth::{
            adapter::{AuthMwCache, AuthMwRepo},
            interceptor::AuthenticationGuard,
        },
        db::models::role::Role,
    };

    pub type AuthenticationMiddleware = AuthenticationGuard<
        AuthMwRepo<Database, SessionAdapter>,
        AuthMwCache<RedisDriver, BasicCache>,
    >;

    impl AuthenticationMiddleware {
        pub fn new(state: &AppState) -> Self {
            Self {
                repo: AuthMwRepo::new(state.pg_diesel.clone(), SessionAdapter),
                cache: AuthMwCache::new(state.redis.clone(), BasicCache),
                min_role: Role::User,
            }
        }
    }
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
        AuthenticationRepositoryAccess<Database, UserAdapter, SessionAdapter, OAuthAdapter>,
        AuthenticationCacheAccess<RedisDriver, BasicCache>,
        Email,
    >;

    impl Configure<AppState> for AuthenticationService {
        fn configure(state: &AppState, cfg: &mut actix_web::web::ServiceConfig) {
            let service = Self {
                repository: AuthenticationRepositoryAccess::new(
                    state.pg_diesel.clone(),
                    UserAdapter,
                    SessionAdapter,
                    OAuthAdapter,
                ),
                cache: AuthenticationCacheAccess::new(state.redis.clone(), BasicCache),
                email: Email::new(
                    state.email.clone(),
                    Email::load_domain_env().expect("Could not load email domain from env"),
                ),
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
        AuthenticationRepositoryAccess<Database, UserAdapter, SessionAdapter, OAuthAdapter>,
        AuthenticationCacheAccess<RedisDriver, BasicCache>,
    >;

    impl Configure<AppState> for OAuthService {
        fn configure(state: &AppState, cfg: &mut actix_web::web::ServiceConfig) {
            let service = Self {
                repository: AuthenticationRepositoryAccess::new(
                    state.pg_diesel.clone(),
                    UserAdapter,
                    SessionAdapter,
                    OAuthAdapter,
                ),
                cache: AuthenticationCacheAccess::new(state.redis.clone(), BasicCache),
            };
            cfg.app_data(web::Data::new(service));
        }
    }
}

pub(super) mod user_service {
    use super::*;
    use crate::app::core::users::{adapters::UsersRepository, domain::Users};

    pub type UserService = Users<UsersRepository<Database, UserAdapter>>;

    impl Configure<AppState> for UserService {
        fn configure(state: &AppState, cfg: &mut actix_web::web::ServiceConfig) {
            let service = Self {
                repository: UsersRepository::new(state.pg_diesel.clone(), UserAdapter),
            };
            cfg.app_data(web::Data::new(service));
        }
    }
}
