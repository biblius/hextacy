use super::{
    data::{Credentials, EmailToken, Otp, RegistrationData, SetPassword},
    service::Authentication,
};
use crate::{
    error::{AuthenticationError, Error},
    models::session::Session,
};
use actix_web::{web, HttpMessage, HttpRequest, Responder};
use tracing::info;
use validator::Validate;

/// Verifies the user's login credentials and either establishes a session if the user
/// doesn't have 2FA or prompts the user for their 2FA pass if they have it set up
pub(super) async fn verify_credentials(
    auth_form: web::Form<Credentials>,
    service: web::Data<Authentication>,
) -> Result<impl Responder, Error> {
    info!("Credentials login : {:?}", auth_form.0);

    auth_form.0.validate().map_err(|e| Error::new(e))?;

    service.verify_credentials(auth_form.0).await
}

/// Verifies the user's OTP if they have 2FA enabled
pub(super) async fn verify_otp(
    otp: web::Form<Otp>,
    service: web::Data<Authentication>,
) -> Result<impl Responder, Error> {
    info!("OTP login : {:?}", otp.0);

    otp.0.validate().map_err(|e| Error::new(e))?;

    service.verify_otp(otp.0).await
}

/// Starts the registration process for the user and sends an email containing a temporary
/// token used to complete the registration
pub(super) async fn start_registration(
    data: web::Form<RegistrationData>,
    service: web::Data<Authentication>,
) -> Result<impl Responder, Error> {
    info!("Start Registration: {:?}", data.0);

    data.0.validate().map_err(|e| Error::new(e))?;

    service.start_registration(data.0).await
}

/// Verifies the user's registration token
pub(super) async fn verify_registration_token(
    token: web::Query<EmailToken>,
    service: web::Data<Authentication>,
) -> Result<impl Responder, Error> {
    info!("Verify registration token: {:?}", token);

    token.0.validate().map_err(|e| Error::new(e))?;

    service.verify_registration_token(token.inner()).await
}

/// Sets the user's password after successful email token verification. Requires a token generated
/// after successful email verification
pub(super) async fn set_password(
    data: web::Form<SetPassword>,
    service: web::Data<Authentication>,
) -> Result<impl Responder, Error> {
    info!("Set password: {:?}", data);

    data.0.validate().map_err(|e| Error::new(e))?;

    service.set_password(data.0).await
}

/// Sets the user's OTP secret. Requires a valid session to be established beforehand
pub(super) async fn set_otp_secret(
    service: web::Data<Authentication>,
    req: HttpRequest,
) -> Result<impl Responder, Error> {
    let extentions = req.extensions();
    let session = extentions.get::<Session>();

    if let Some(session) = session {
        info!("Registering OTP secret for: {}", session.user_id);

        service.set_otp_secret(&session.user_id).await
    } else {
        Err(AuthenticationError::SessionNotFound.into())
    }
}
