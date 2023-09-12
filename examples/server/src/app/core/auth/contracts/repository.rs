use crate::db::dto::oauth::OAuthMetaData;
use crate::db::models::oauth::OAuthMeta;
use crate::db::models::{session, user};
use crate::db::repository::oauth::OAuthRepository;
use crate::db::repository::session::SessionRepository;
use crate::db::repository::user::UserRepository;
use crate::db::RepoAdapterError;
use crate::error::Error;
use crate::services::oauth::{OAuthProvider, OAuthTokenResponse};
use hextacy::{component, contract, info};

#[component(
    use Driver as driver,
    use
      UserRepository,
      SessionRepository,
      OAuthRepository,
)]
pub struct AuthenticationRepositoryAccess {}

#[component(
    use Driver for Connection:Atomic,
    use UserRepository with Connection as User,
    use SessionRepository with Connection as Session,
)]
#[contract]
impl<OAuth> AuthenticationRepositoryAccess<OAuth>
where
    // Just to try out component with existing generics
    OAuth:
        OAuthRepository<Connection> + OAuthRepository<Connection::TransactionResult> + Send + Sync,
{
    async fn get_user_by_id(&self, id: &str) -> Result<user::User, Error> {
        let mut conn = self.driver.connect().await?;
        User::get_by_id(&mut conn, id).await.map_err(Error::new)
    }

    async fn get_user_by_email(&self, email: &str) -> Result<user::User, Error> {
        let mut conn = self.driver.connect().await?;
        User::get_by_email(&mut conn, email)
            .await
            .map_err(Error::new)
    }

    async fn create_user(
        &self,
        email: &str,
        username: &str,
        pw: &str,
    ) -> Result<user::User, Error> {
        let mut conn = self.driver.connect().await?;
        User::create(&mut conn, email, username, pw)
            .await
            .map_err(Error::new)
    }

    async fn update_user_email_verification(&self, id: &str) -> Result<user::User, Error> {
        let mut conn = self.driver.connect().await?;
        User::update_email_verified_at(&mut conn, id)
            .await
            .map_err(Error::new)
    }

    async fn update_user_otp_secret(&self, id: &str, secret: &str) -> Result<user::User, Error> {
        let mut conn = self.driver.connect().await?;
        User::update_otp_secret(&mut conn, id, secret)
            .await
            .map_err(Error::new)
    }

    async fn update_user_password(&self, id: &str, hashed_pw: &str) -> Result<user::User, Error> {
        let mut conn = self.driver.connect().await?;
        User::update_password(&mut conn, id, hashed_pw)
            .await
            .map_err(Error::new)
    }

    async fn freeze_user(&self, id: &str) -> Result<user::User, Error> {
        let mut conn = self.driver.connect().await?;
        User::freeze(&mut conn, id).await.map_err(Error::new)
    }

    async fn create_session<'a>(
        &self,
        user: &user::User,
        csrf: &str,
        expires: Option<i64>,
        access_token: Option<&'a str>,
        provider: Option<OAuthProvider>,
    ) -> Result<session::Session, Error> {
        let mut conn = self.driver.connect().await?;
        Session::create(&mut conn, user, csrf, expires, access_token, provider)
            .await
            .map_err(Error::new)
    }

    async fn expire_session(&self, id: &str) -> Result<session::Session, Error> {
        let mut conn = self.driver.connect().await?;
        Session::expire(&mut conn, id).await.map_err(Error::new)
    }

    async fn purge_sessions<'a>(
        &self,
        user_id: &str,
        skip: Option<&'a str>,
    ) -> Result<Vec<session::Session>, Error> {
        let mut conn = self.driver.connect().await?;
        Session::purge(&mut conn, user_id, skip)
            .await
            .map_err(Error::new)
    }

    async fn get_or_create_user_oauth(
        &self,
        account_id: &str,
        email: &str,
        username: &str,
        tokens: &OAuthTokenResponse,
        provider: OAuthProvider,
    ) -> Result<(user::User, OAuthMeta), Error> {
        let conn = self.driver.connect().await?;
        let mut conn = conn.start_transaction().await?;

        let user = match User::get_by_email(&mut conn, email).await {
            Ok(user) => User::update_oauth_id(&mut conn, &user.id, account_id, provider)
                .await
                .map_err(Error::new)?,
            Err(RepoAdapterError::DoesNotExist) => {
                User::create_from_oauth(&mut conn, account_id, email, username, provider)
                    .await
                    .map_err(Error::new)?
            }
            Err(e) => {
                Connection::abort_transaction(conn).await?;
                return Err(Error::new(e));
            }
        };

        let existing_oauth = match OAuth::get_by_account_id(&mut conn, account_id).await {
            Ok(oauth) => oauth,
            Err(e) => match e {
                // If the entry does not exist, we must create one for the user
                RepoAdapterError::DoesNotExist => {
                    info!("OAuth entry does not exist, creating");
                    let data = OAuthMetaData {
                        user_id: &user.id,
                        access_token: &tokens.access_token,
                        refresh_token: tokens.refresh_token.as_deref(),
                        provider,
                        scope: &tokens.scope,
                        account_id: Some(account_id),
                    };
                    OAuth::create(&mut conn, data).await.map_err(Error::new)?
                }
                e => {
                    Connection::abort_transaction(conn).await?;
                    return Err(Error::new(e));
                }
            },
        };

        Connection::commit_transaction(conn).await?;

        Ok((user, existing_oauth))
    }

    async fn get_oauth_by_account_id(&self, account_id: &str) -> Result<OAuthMeta, Error> {
        let mut conn = self.driver.connect().await?;
        OAuth::get_by_account_id(&mut conn, account_id)
            .await
            .map_err(Error::new)
    }

    async fn refresh_oauth_and_session(
        &self,
        user_id: &str,
        tokens: &OAuthTokenResponse,
        provider: OAuthProvider,
    ) -> Result<(), Error> {
        self.update_oauth(user_id, tokens, provider).await?;
        self.update_session_access_tokens(&tokens.access_token, user_id, provider)
            .await?;
        Ok(())
    }

    async fn update_oauth(
        &self,
        user_id: &str,
        tokens: &OAuthTokenResponse,
        provider: OAuthProvider,
    ) -> Result<OAuthMeta, Error> {
        let mut conn = self.driver.connect().await?;
        let data = OAuthMetaData {
            user_id,
            access_token: &tokens.access_token,
            refresh_token: tokens.refresh_token.as_deref(),
            provider,
            scope: &tokens.scope,
            account_id: None,
        };
        OAuth::update(&mut conn, data).await.map_err(Error::new)
    }

    async fn update_session_access_tokens(
        &self,
        access_token: &str,
        user_id: &str,
        provider: OAuthProvider,
    ) -> Result<Vec<session::Session>, Error> {
        let mut conn = self.driver.connect().await?;
        Session::update_access_tokens(&mut conn, access_token, user_id, provider)
            .await
            .map_err(Error::new)
    }
}
/* implement! {
    AuthenticationRepositoryAccess : RepositoryApi,

    use Postgres for Connection : Atomic;

    User    as UserRepository<Connection>,
    Session as SessionRepository<Connection>,
    OAuth   as OAuthRepository<Connection>;


} */

/* #[async_trait]
impl<Pg, Mg, Connection, MgConn, User, Session, OAuth> RepositoryApi
    for AuthenticationRepositoryAccess<Pg, Mg, Connection, MgConn, User, Session, OAuth>
where
    Self: RepositoryAccess<Connection> + RepositoryAccess<MgConn>,
    Pg: Driver<Connection = Connection>,
    Mg: Driver<Connection = MgConn>,
    User: UserRepository<MgConn>,
    Session: SessionRepository<Connection>,
    OAuth: OAuthRepository<Connection>,
{
}
 */
/*

impl<Pg, Mg, Connection, MgConn, User, Session, OAuth>
    Repository<Pg, Mg, Connection, MgConn, User, Session, OAuth>
where
    Pg: Driver<Connection = Connection>,
    Mg: Driver<Connection = MgConn>,
    User: UserRepository<MgConn>,
    Session: SessionRepository<Connection>,
    OAuth: OAuthRepository<Connection>,
{
    pub fn new(pg_driver: Arc<Pg>, mg_driver: Arc<Mg>) -> Self {
        Self {
            postgres: Driver::new(pg_driver),
            mongo: Driver::new(mg_driver),
            user: PhantomData,
            session: PhantomData,
            oauth: PhantomData,
        }
    }
}
*/

/* #[derive(Debug, hextacy::derive::Repository)]
pub struct AuthenticationRepositoryAccess<Pg, Connection, User, Session, OAuth>
where
    Pg: Driver<Connection = Connection>,
{
    #[diesel(Connection)]
    postgres: hextacy::drivers::db::Driver<Pg, Connection>,
    user: std::marker::PhantomData<User>,
    session: std::marker::PhantomData<Session>,
    oauth: std::marker::PhantomData<OAuth>,
} */
