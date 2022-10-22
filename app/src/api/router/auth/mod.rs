pub(super) mod contract;
pub(super) mod data;
pub(super) mod domain;
pub(super) mod handler;
pub(super) mod infrastructure;
pub(in super::super) mod setup;

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{
        contract::{MockCacheContract, MockEmailContract, MockServiceContract, ServiceContract},
        data::{AuthenticationSuccessResponse, Credentials, Otp, RegistrationData},
        domain::Authentication,
    };
    use crate::{api::router::auth::contract::MockRepositoryContract, error::Error};
    use actix_web::body::to_bytes;
    use data_encoding::BASE32;
    use derive_new::new;
    use infrastructure::{
        adapters::postgres::PgAdapterError,
        config::env,
        crypto::utility::{bcrypt_hash, uuid},
        repository::{session::Session, user::User},
        utility::http::response::Response,
    };
    use lazy_static::lazy_static;
    use reqwest::StatusCode;
    use serde::{Deserialize, Serialize};

    /// Mock this one here so we don't clog the code with unnecessary implementations
    #[derive(Debug, Serialize, Deserialize, new)]
    struct TwoFactorAuthResponse {
        pub username: String,
        pub token: String,
        pub remember: bool,
    }

    lazy_static! {
        static ref CREDENTIALS: Credentials = Credentials {
            email: "test@lo.com".to_string(),
            password: "123".to_string(),
            remember: false,
        };
        static ref REGISTRATION: RegistrationData = RegistrationData {
            email: "test@lo.com".to_string(),
            password: "123".to_string(),
            username: "bibli".to_string(),
        };
        static ref USER_NO_OTP: User = User::__mock(
            uuid(),
            "bibli@khan.com",
            "bibli",
            bcrypt_hash("123").unwrap(),
            false,
            true,
            false
        );
        static ref USER_OTP: User = User::__mock(
            uuid(),
            "bibli@khan.com",
            "bibli",
            bcrypt_hash("123").unwrap(),
            true,
            true,
            false
        );
        static ref SESSION_NO_OTP: Session = Session::__mock(uuid(), &USER_NO_OTP, uuid(), false);
        static ref SESSION_OTP: Session = Session::__mock(uuid(), &USER_OTP, uuid(), true);
    }

    #[actix_web::main]
    #[test]
    async fn registration() {
        env::load_from_file("../.env").unwrap();

        let mut repository = MockRepositoryContract::new();

        // The service will first attempt to find an existing user
        repository
            .expect_get_user_by_email()
            .return_once_st(move |_| {
                Err(Error::new(PgAdapterError::DoesNotExist(format!(
                    "User ID: {}",
                    USER_NO_OTP.clone().id
                ))))
            });

        // Then create one
        repository
            .expect_create_user()
            .return_once(move |_, _, _| Ok(USER_NO_OTP.clone()));

        // Cache their registration token
        let mut cache = MockCacheContract::new();
        cache
            .expect_set_token()
            .return_once_st(move |_, _, _: &String, _| Ok(()));

        // And send it via email
        let mut email = MockEmailContract::new();
        email
            .expect_send_registration_token()
            .return_once_st(move |_, _, _| Ok(()));

        let auth_service = Authentication {
            repository,
            cache,
            email,
        };

        auth_service
            .start_registration(REGISTRATION.clone())
            .await
            .unwrap();
    }

    #[actix_web::main]
    #[test]
    async fn credentials_no_otp() {
        env::load_from_file("../.env").unwrap();

        let mut service = MockServiceContract::new();
        let mut repository = MockRepositoryContract::new();
        let mut cache = MockCacheContract::new();
        let email = MockEmailContract::new();

        // Find user without OTP secret
        repository
            .expect_get_user_by_email()
            .return_once(move |_| Ok(USER_NO_OTP.clone()));

        // Create session
        repository
            .expect_create_session()
            .return_once(move |_, _, _| Ok(SESSION_NO_OTP.clone()));

        // Delete login attempts
        cache.expect_delete_login_attempts().return_once(|_| Ok(()));

        // Respond with session
        service
            .expect_session_response()
            .return_once_st(move |_, _| {
                Ok(
                    AuthenticationSuccessResponse::new(USER_NO_OTP.clone(), SESSION_NO_OTP.clone())
                        .to_response(StatusCode::OK, None, None),
                )
            });

        let auth = Authentication {
            repository,
            cache,
            email,
        };

        auth.verify_credentials(CREDENTIALS.clone()).await.unwrap();
    }

    #[actix_web::main]
    #[test]
    async fn credentials_and_otp() {
        env::load_from_file("../.env").unwrap();

        let mut repository = MockRepositoryContract::new();
        let mut cache = MockCacheContract::new();
        let email = MockEmailContract::new();

        // Expect the user to exist
        repository
            .expect_get_user_by_email()
            .return_once(move |_| Ok(USER_OTP.clone()));

        // Expect to cache an OTP token
        cache
            .expect_set_token()
            .return_once(move |_, _, _: &String, _| Ok(()));

        let auth = Authentication {
            repository,
            cache,
            email,
        };

        // Verify the creds and grab the token from the response
        let res = auth.verify_credentials(CREDENTIALS.clone()).await.unwrap();
        let body = to_bytes(res.into_body()).await.unwrap();
        let token =
            serde_json::from_str::<TwoFactorAuthResponse>(std::str::from_utf8(&body).unwrap())
                .unwrap()
                .token;

        let mut repository = MockRepositoryContract::new();
        let mut cache = MockCacheContract::new();
        let email = MockEmailContract::new();

        // Get the OTP token
        cache
            .expect_get_token()
            .returning(move |_, _| Ok(USER_OTP.id.clone()));

        // Get the user's ID stored behind the token
        repository
            .expect_get_user_by_id()
            .returning(move |_| Ok(USER_OTP.clone()));

        // Delete the OTP token
        cache.expect_delete_token().return_once(move |_, _| Ok(()));

        // Create a session
        repository
            .expect_create_session()
            .returning(move |_, _, _| Ok(SESSION_OTP.clone()));

        // Delete login attempts
        cache.expect_delete_login_attempts().return_once(|_| Ok(()));

        // Cache the session since it has the permanent flag enabled
        cache.expect_set_session().return_once(move |_, _| Ok(()));

        let auth = Authentication {
            repository,
            cache,
            email,
        };

        // Grab current time slice and calculate the OTP
        let time_step_now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            / 30;
        let sec = USER_OTP.otp_secret.clone();
        let otp = thotp::otp(
            &BASE32.decode(sec.clone().unwrap().as_bytes()).unwrap(),
            time_step_now,
        )
        .unwrap();

        let data = Otp {
            password: otp,
            token,
            remember: true,
        };

        auth.verify_otp(data).await.unwrap();
    }
}
