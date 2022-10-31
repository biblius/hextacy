use super::{
    domain::Authentication,
    handler,
    infrastructure::{Cache, Email, Repository},
};
use crate::api::middleware::auth::interceptor;
use actix_web::web::{self, Data};
use infrastructure::{
    clients::{
        email::lettre::SmtpTransport,
        store::{postgres::Postgres, redis::Redis},
    },
    store::adapters::postgres::{session::PgSessionAdapter, user::PgUserAdapter},
    store::repository::role::Role,
};
use std::sync::Arc;

pub(crate) fn routes(
    pg: Arc<Postgres>,
    rd: Arc<Redis>,
    email: Arc<SmtpTransport>,
    cfg: &mut web::ServiceConfig,
) {
    let service = Authentication {
        repository: Repository {
            user_repo: PgUserAdapter { client: pg.clone() },
            session_repo: PgSessionAdapter { client: pg.clone() },
        },
        cache: Cache { client: rd.clone() },
        email: Email { client: email },
    };
    let guard = interceptor::AuthGuard::new(pg, rd, Role::User);
    cfg.app_data(Data::new(service));
    cfg.service(
        web::scope("/auth")
            .service(
                web::resource("/login").route(web::post().to(handler::login::<
                    Authentication<Repository<PgUserAdapter, PgSessionAdapter>, Cache, Email>,
                >)),
            )
            .service(web::resource("/register").route(web::post().to(
                handler::start_registration::<
                    Authentication<Repository<PgUserAdapter, PgSessionAdapter>, Cache, Email>,
                >,
            )))
            .service(
                web::resource("/verify-registration-token").route(web::get().to(
                    handler::verify_registration_token::<
                        Authentication<Repository<PgUserAdapter, PgSessionAdapter>, Cache, Email>,
                    >,
                )),
            )
            .service(
                web::resource("/resend-registration-token").route(web::post().to(
                    handler::resend_registration_token::<
                        Authentication<Repository<PgUserAdapter, PgSessionAdapter>, Cache, Email>,
                    >,
                )),
            )
            .service(
                web::resource("/set-otp")
                    .route(web::get().to(handler::set_otp_secret::<
                        Authentication<Repository<PgUserAdapter, PgSessionAdapter>, Cache, Email>,
                    >))
                    .wrap(guard.clone()),
            )
            .service(
                web::resource("/verify-otp").route(web::post().to(handler::verify_otp::<
                    Authentication<Repository<PgUserAdapter, PgSessionAdapter>, Cache, Email>,
                >)),
            )
            .service(
                web::resource("/change-password")
                    .route(web::post().to(handler::change_password::<
                        Authentication<Repository<PgUserAdapter, PgSessionAdapter>, Cache, Email>,
                    >))
                    .wrap(guard.clone()),
            )
            .service(web::resource("/forgot-password").route(web::post().to(
                handler::forgot_password::<
                    Authentication<Repository<PgUserAdapter, PgSessionAdapter>, Cache, Email>,
                >,
            )))
            .service(
                web::resource("/verify-forgot-password").route(web::post().to(
                    handler::verify_forgot_password::<
                        Authentication<Repository<PgUserAdapter, PgSessionAdapter>, Cache, Email>,
                    >,
                )),
            )
            .service(web::resource("/reset-password").route(web::get().to(
                handler::reset_password::<
                    Authentication<Repository<PgUserAdapter, PgSessionAdapter>, Cache, Email>,
                >,
            )))
            .service(
                web::resource("/logout")
                    .route(web::post().to(handler::logout::<
                        Authentication<Repository<PgUserAdapter, PgSessionAdapter>, Cache, Email>,
                    >))
                    .wrap(guard),
            ),
    );
}
