use super::{contract::ServiceContract, data::OAuthCodeExchange};
use crate::{
    api::router::auth::{
        contract::{CacheContract, RepoContract},
        data::AuthenticationSuccessResponse,
    },
    config::constants::COOKIE_S_ID,
    error::{AuthenticationError, Error},
};
use actix_web::HttpResponse;
use alx_core::{
    clients::oauth::{OAuth, OAuthAccount, TokenResponse},
    crypto::uuid,
    web::http::{
        cookie,
        response::{MessageResponse, Response},
    },
};
use async_trait::async_trait;
use reqwest::{
    header::{HeaderName, HeaderValue},
    StatusCode,
};
use storage::{
    adapters::AdapterError,
    models::{session::Session, user::User},
};
use tracing::info;

pub(super) struct OAuthService<P, R, C>
where
    P: OAuth,
    R: RepoContract,
    C: CacheContract,
{
    pub provider: P,
    pub repo: R,
    pub cache: C,
}

#[async_trait(?Send)]
impl<P, R, C> ServiceContract for OAuthService<P, R, C>
where
    P: OAuth + Send + Sync,
    R: RepoContract,
    C: CacheContract + Send + Sync,
{
    async fn login(&self, code: OAuthCodeExchange) -> Result<HttpResponse, Error> {
        let OAuthCodeExchange { ref code } = code;

        // Get the tokens and obtain the account
        let mut tokens = self.provider.exchange_code(code).await?;
        let account = self.provider.get_account(&tokens).await?;
        let provider = self.provider.provider_id();
        let email = match account.email() {
            Some(email) => email,
            None => return Err(AuthenticationError::EmailUnverified.into()),
        };

        // let mut trx = self.repo.transaction()?;

        let user =
            match self.repo.get_user_by_email(&email) {
                Ok(user) => self
                    .repo
                    .update_user_provider_id(&user.id, &account.id(), provider)?,
                Err(Error::Adapter(AdapterError::DoesNotExist)) => self
                    .repo
                    .create_user_from_oauth(&account.id(), &email, account.username(), provider)?,
                Err(e) => {
                    //trx.rollback()?;
                    return Err(e.into());
                }
            };

        let existing_oauth = match self.repo.get_oauth_by_account_id(&account.id()) {
            Ok(oauth) => oauth,
            Err(e) => match e {
                // If the entry does not exist, we must create one for the user
                Error::Adapter(AdapterError::DoesNotExist) => {
                    info!("OAuth entry does not exist, creating");
                    self.repo
                        .create_oauth(&user.id, &account.id(), &tokens, provider)?
                }
                e => {
                    //trx.rollback()?;
                    return Err(e.into());
                }
            },
        };

        if existing_oauth.expired() {
            // If refresh token exists, refresh and update
            if let Some(ref refresh_token) = existing_oauth.refresh_token {
                info!("OAuth access token expired, refreshing");
                tokens = self.provider.refresh_access_token(refresh_token).await?;

                self.repo.update_oauth(&user.id, &tokens, provider)?;
                self.repo.update_session_access_tokens(
                    tokens.access_token(),
                    &user.id,
                    provider,
                )?;
            // Otherwise just update the existing entry
            } else {
                self.repo.update_oauth(&user.id, &tokens, provider)?;
            }
        }

        // trx.commit()?;

        Ok(MessageResponse::new("l")
            .to_response(StatusCode::OK)
            .finish())
    }

    fn register<TR, A>(&self, tokens: TR, account: A) -> Result<HttpResponse, Error>
    where
        TR: TokenResponse + 'static,
        A: OAuthAccount + 'static,
    {
        let provider = self.provider.provider_id();

        info!(
            "Registering new OAuth entry with provider {} and id {}",
            provider,
            &account.id()
        );

        let email = match account.email() {
            Some(email) => email,
            None => return Err(AuthenticationError::EmailUnverified.into()),
        };

        // Check if the user already exists under a different provider and update their entry if they do,
        // otherwise create
        let user =
            match self.repo.get_user_by_email(email) {
                Ok(user) => self
                    .repo
                    .update_user_provider_id(&user.id, &account.id(), provider)?,
                Err(Error::Adapter(AdapterError::DoesNotExist)) => self
                    .repo
                    .create_user_from_oauth(&account.id(), email, account.username(), provider)?,
                Err(e) => return Err(e.into()),
            };

        self.repo
            .create_oauth(&user.id, &account.id(), &tokens, provider)?;

        self.establish_session(tokens, user)
    }

    async fn request_additional_scopes(
        &self,
        mut session: Session,
        code: OAuthCodeExchange,
    ) -> Result<HttpResponse, Error> {
        let _ = session
            .oauth_token
            .ok_or_else(|| AuthenticationError::InvalidToken("OAuth"))?;

        // Obtain the new tokens with more scopes
        let OAuthCodeExchange { ref code } = code;
        let tokens = self.provider.exchange_code(code).await?;

        let user_id = &session.user_id;
        let provider = self.provider.provider_id();
        let access_token = tokens.access_token();

        // Update existing sessions tied to the user and their auth provider
        // as well as the related oauth metadata
        self.repo
            .update_session_access_tokens(access_token, user_id, provider)?;
        self.repo.update_oauth(user_id, &tokens, provider)?;

        // Update the existing session, sessions updated in the previous step will not update
        // cached sessions so we have to cache the current one to reflect the change
        session.oauth_token = Some(access_token.to_string());

        self.cache.set_session(&session.id, &session)?;

        Ok(MessageResponse::new("lol")
            .to_response(StatusCode::OK)
            .finish())
    }

    fn establish_session<TR: TokenResponse>(
        &self,
        tokens: TR,
        user: User,
    ) -> Result<HttpResponse, Error> {
        let csrf_token = uuid();

        let expiration = tokens.expires_in();
        let access_token = tokens.access_token();

        let session = self.repo.create_session(
            &user,
            &csrf_token,
            expiration,
            Some(access_token),
            Some(self.provider.provider_id()),
        )?;

        let session_cookie = cookie::create(COOKIE_S_ID, &session.id, false)?;

        // Cache the session
        self.cache.set_session(&session.id, &session)?;

        info!("Successfully created session for {}", user.username);

        // Respond with the x-csrf header and the session ID
        Ok(AuthenticationSuccessResponse::new(user)
            .to_response(StatusCode::OK)
            .with_cookies(vec![session_cookie])
            .with_headers(vec![(
                HeaderName::from_static("x-csrf-token"),
                HeaderValue::from_str(&csrf_token)?,
            )])
            .finish())
    }
}
