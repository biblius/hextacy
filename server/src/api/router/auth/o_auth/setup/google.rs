use super::super::{domain::OAuthService, handler};
use crate::api::{
    middleware::auth::interceptor::AuthGuard,
    router::auth::adapter::{Cache, Repository},
};
use actix_web::web::{self, Data};
use alx_core::clients::{
    db::{
        postgres::{PgPoolConnection, Postgres},
        redis::Redis,
    },
    oauth::google::GoogleOAuth,
};
use std::{cell::RefCell, sync::Arc};
use storage::{
    adapters::postgres::{oauth::PgOAuthAdapter, session::PgSessionAdapter, user::PgUserAdapter},
    models::role::Role,
};

pub(crate) fn routes(pg: Arc<Postgres>, rd: Arc<Redis>, cfg: &mut web::ServiceConfig) {
    let service = OAuthService {
        provider: GoogleOAuth,
        repo: Repository {
            client: pg.clone(),
            trx: Option::<RefCell<PgPoolConnection>>::None,
            _user: PgUserAdapter,
            _session: PgSessionAdapter,
            _oauth: PgOAuthAdapter,
        },
        cache: Cache { client: rd.clone() },
    };

    let auth_guard = AuthGuard::new(pg, rd, Role::User);

    cfg.app_data(Data::new(service));

    cfg.service(
        web::resource("/auth/oauth/google/login").route(web::post().to(handler::login::<
            OAuthService<
                GoogleOAuth,
                Repository<PgUserAdapter, PgSessionAdapter, PgOAuthAdapter, PgPoolConnection>,
                Cache,
            >,
        >)),
    );

    cfg.service(
        web::resource("/auth/oauth/google/scope")
            .route(web::put().to(handler::request_scopes::<
                OAuthService<
                    GoogleOAuth,
                    Repository<PgUserAdapter, PgSessionAdapter, PgOAuthAdapter, PgPoolConnection>,
                    Cache,
                >,
            >))
            .wrap(auth_guard),
    );
}
