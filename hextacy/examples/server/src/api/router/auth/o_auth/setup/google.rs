use super::super::{handler, service::OAuthService};
use crate::db::{
    adapters::postgres::{oauth::PgOAuthAdapter, session::PgSessionAdapter, user::PgUserAdapter},
    models::role::Role,
};
use crate::{
    api::{
        middleware::auth::interceptor::AuthGuard,
        router::auth::adapter::{Cache, Repository},
    },
    services::oauth::google::GoogleOAuth,
};
use actix_web::web::{self, Data};
use hextacy::clients::{
    cache::redis::Redis,
    db::postgres::{PgPoolConnection, Postgres},
};
use std::sync::Arc;

pub(crate) fn routes(pg: Arc<Postgres>, rd: Arc<Redis>, cfg: &mut web::ServiceConfig) {
    let service = OAuthService {
        provider: GoogleOAuth,
        repository: Repository::<
            Postgres,
            PgPoolConnection,
            PgUserAdapter,
            PgSessionAdapter,
            PgOAuthAdapter,
        >::new(pg.clone()),
        cache: Cache { client: rd.clone() },
    };

    let auth_guard = AuthGuard::new(pg, rd, Role::User);

    cfg.app_data(Data::new(service));

    cfg.service(
        web::resource("/auth/oauth/google/login").route(web::post().to(handler::login::<
            OAuthService<
                GoogleOAuth,
                Repository<
                    Postgres,
                    PgPoolConnection,
                    PgUserAdapter,
                    PgSessionAdapter,
                    PgOAuthAdapter,
                >,
                Cache,
            >,
        >)),
    );

    cfg.service(
        web::resource("/auth/oauth/google/scope")
            .route(web::put().to(handler::request_scopes::<
                OAuthService<
                    GoogleOAuth,
                    Repository<
                        Postgres,
                        PgPoolConnection,
                        PgUserAdapter,
                        PgSessionAdapter,
                        PgOAuthAdapter,
                    >,
                    Cache,
                >,
            >))
            .wrap(auth_guard),
    );
}
