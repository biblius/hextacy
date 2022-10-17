pub mod data;
pub mod handler;
pub mod response;
pub mod service;

use self::service::email::Email;
use actix_web::web::{self, Data};
use infrastructure::{
    email::lettre::SmtpTransport,
    storage::{postgres::Pg, redis::Rd},
};
use service::{authentication::Authentication, cache::Cache, postgres::Postgres};
use std::sync::Arc;

pub fn init(
    pg_pool: Arc<Pg>,
    rd_pool: Arc<Rd>,
    email_client: Arc<SmtpTransport>,
    cfg: &mut web::ServiceConfig,
) {
    let service = Authentication {
        database: Postgres::new(pg_pool),
        cache: Cache::new(rd_pool),
        email: Email::new(email_client),
    };

    cfg.app_data(Data::new(service));

    // Credentials login
    cfg.service(
        web::resource("/auth/login/credentials").route(web::post().to(handler::login::credentials)),
    );

    // OTP login
    cfg.service(web::resource("/auth/login/otp").route(web::post().to(handler::login::otp)));

    // Initial registration
    cfg.service(
        web::resource("/auth/register")
            .route(web::post().to(handler::registration::start_registration)),
    );

    // Verify registration token
    cfg.service(
        web::resource("/auth/verify-registration-token")
            .route(web::get().to(handler::registration::verify_registration_token)),
    );

    // Set password
    cfg.service(
        web::resource("/auth/{user_id}/set-password")
            .route(web::post().to(handler::registration::set_password)),
    );
}
