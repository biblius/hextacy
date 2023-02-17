use super::super::{domain::OAuthService, handler};
use crate::api::{middleware::auth::interceptor::AuthGuard, router::auth::infrastructure::Cache};
use actix_web::web::{self, Data};
use alx_core::clients::{
    db::{postgres::Postgres, redis::Redis},
    oauth::google::GoogleOAuth,
};
use std::sync::Arc;
use storage::{
    adapters::postgres::{oauth::PgOAuthAdapter, session::PgSessionAdapter, user::PgUserAdapter},
    models::role::Role,
};

pub(crate) fn routes(pg: Arc<Postgres>, rd: Arc<Redis>, cfg: &mut web::ServiceConfig) {
    let service = OAuthService {
        provider: GoogleOAuth,
        user_repo: PgUserAdapter { client: pg.clone() },
        session_repo: PgSessionAdapter { client: pg.clone() },
        oauth_repo: PgOAuthAdapter { client: pg.clone() },
        cache: Cache { client: rd.clone() },
    };

    let auth_guard = AuthGuard::new(pg, rd, Role::User);

    cfg.app_data(Data::new(service));

    cfg.service(
        web::resource("/auth/oauth/google/login").route(web::post().to(handler::login::<
            OAuthService<GoogleOAuth, PgUserAdapter, PgSessionAdapter, PgOAuthAdapter, Cache>,
        >)),
    );

    cfg.service(
        web::resource("/auth/oauth/google/scope")
            .route(web::put().to(handler::request_scopes::<
                OAuthService<GoogleOAuth, PgUserAdapter, PgSessionAdapter, PgOAuthAdapter, Cache>,
            >))
            .wrap(auth_guard),
    );
}
