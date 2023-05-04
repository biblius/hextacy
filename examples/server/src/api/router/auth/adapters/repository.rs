use crate::db::adapters::AdapterError;
use crate::db::models::oauth::OAuthMeta;
use crate::db::models::{session, user};
use crate::db::repository::oauth::OAuthRepository;
use crate::db::repository::session::SessionRepository;
use crate::db::repository::user::UserRepository;
use crate::error::Error;
use crate::services::oauth::{OAuthProvider, TokenResponse};
use hextacy::db::{Atomic, RepositoryAccess};

use hextacy::adapt;
#[allow(unused_imports)]
use hextacy::drivers::db::postgres::diesel::{PgPoolConnection, PostgresDiesel};
use hextacy::drivers::db::DBConnect;
use sea_orm::DatabaseConnection;
use tracing::info;

adapt! {
    ServiceAdapter in crate::api::router::auth,

    use Postgres for Connection:Atomic as driver : seaorm;

    UserRepository<Connection:Atomic> as User,
    SessionRepository<Connection> as Session,
    OAuthRepository<Connection> as OAuth;

    async fn get_user_by_id(&self, id: &str) -> Result<user::User, Error> {
        let mut conn = self.connect().await?;
        User::get_by_id(&mut conn, id).await.map_err(Error::new)
    }

    async fn get_user_by_email(&self, email: &str) -> Result<user::User, Error> {
        let mut conn = self.connect().await?;
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
        let mut conn = self.connect().await?;
        User::create(&mut conn, email, username, pw)
            .await
            .map_err(Error::new)
    }

    async fn update_user_email_verification(&self, id: &str) -> Result<user::User, Error> {
        let mut conn = self.connect().await?;
        User::update_email_verified_at(&mut conn, id)
            .await
            .map_err(Error::new)
    }

    async fn update_user_otp_secret(&self, id: &str, secret: &str) -> Result<user::User, Error> {
        let mut conn = self.connect().await?;
        User::update_otp_secret(&mut conn, id, secret)
            .await
            .map_err(Error::new)
    }

    async fn update_user_password(&self, id: &str, hashed_pw: &str) -> Result<user::User, Error> {
        let mut conn = self.connect().await?;
        User::update_password(&mut conn, id, hashed_pw)
            .await
            .map_err(Error::new)
    }

    async fn freeze_user(&self, id: &str) -> Result<user::User, Error> {
        let mut conn = self.connect().await?;
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
        let mut conn = self.connect().await?;
        Session::create(&mut conn, user, csrf, expires, access_token, provider)
            .await
            .map_err(Error::new)
    }

    async fn expire_session(&self, id: &str) -> Result<session::Session, Error> {
        let mut conn = self.connect().await?;
        Session::expire(&mut conn, id).await.map_err(Error::new)
    }

    async fn purge_sessions<'a>(
        &self,
        user_id: &str,
        skip: Option<&'a str>,
    ) -> Result<Vec<session::Session>, Error> {
        let mut conn = self.connect().await?;
        Session::purge(&mut conn, user_id, skip)
            .await
            .map_err(Error::new)
    }

    async fn get_or_create_user_oauth<T: TokenResponse + 'static>(
        &self,
        account_id: &str,
        email: &str,
        username: &str,
        tokens: &T,
        provider: OAuthProvider,
    ) -> Result<(user::User, OAuthMeta), Error> {
        let conn = self.connect().await?;
        let mut conn = conn.start_transaction().await?;
        let user = match self.get_user_by_email(email).await {
            Ok(user) => User::update_oauth_id(&mut conn, &user.id, account_id, provider)
                .await
                .map_err(Error::new)?,
            Err(Error::Adapter(AdapterError::DoesNotExist)) => {
                self.create_user_from_oauth(account_id, email, username, provider)
                    .await?
            }
            Err(e) => {
                <Connection as Atomic>::abort_transaction(conn).await?;
                return Err(e);
            }
        };

        let existing_oauth = match self.get_oauth_by_account_id(account_id).await {
            Ok(oauth) => oauth,
            Err(e) => match e {
                // If the entry does not exist, we must create one for the user
                Error::Adapter(AdapterError::DoesNotExist) => {
                    info!("OAuth entry does not exist, creating");
                    self.create_oauth(&user.id, account_id, tokens, provider)
                        .await?
                }
                e => {
                    <Connection as Atomic>::abort_transaction(conn).await?;
                    return Err(e);
                }
            },
        };

        <Connection as Atomic>::commit_transaction(conn).await?;

        Ok((user, existing_oauth))
    }

    async fn create_user_from_oauth(
        &self,
        account_id: &str,
        email: &str,
        username: &str,
        provider: OAuthProvider,
    ) -> Result<user::User, Error> {
        let mut conn = self.connect().await?;

        User::create_from_oauth(&mut conn, account_id, email, username, provider)
            .await
            .map_err(Error::new)
    }

    async fn get_oauth_by_account_id(&self, account_id: &str) -> Result<OAuthMeta, Error> {
        let mut conn = self.connect().await?;
        OAuth::get_by_account_id(&mut conn, account_id)
            .await
            .map_err(Error::new)
    }

    async fn create_oauth<T: TokenResponse + 'static>(
        &self,
        user_id: &str,
        account_id: &str,
        tokens: &T,
        provider: OAuthProvider,
    ) -> Result<OAuthMeta, Error> {
        let mut conn = self.connect().await?;
        OAuth::create(&mut conn, user_id, account_id, tokens, provider)
            .await
            .map_err(Error::new)
    }

    async fn refresh_oauth_and_session<T: TokenResponse + 'static>(
        &self,
        user_id: &str,
        tokens: &T,
        provider: OAuthProvider,
    ) -> Result<(), Error> {
        self.update_oauth(user_id, tokens, provider).await?;
        self.update_session_access_tokens(tokens.access_token(), user_id, provider)
            .await?;
        Ok(())
    }

    async fn update_oauth<T: TokenResponse + 'static>(
        &self,
        user_id: &str,
        tokens: &T,
        provider: OAuthProvider,
    ) -> Result<OAuthMeta, Error> {
        let mut conn = self.connect().await?;
        OAuth::update(&mut conn, user_id, tokens, provider)
            .await
            .map_err(Error::new)
    }

    async fn update_session_access_tokens(
        &self,
        access_token: &str,
        user_id: &str,
        provider: OAuthProvider,
    ) -> Result<Vec<session::Session>, Error> {
        let mut conn = self.connect().await?;
        Session::update_access_tokens(&mut conn, access_token, user_id, provider)
            .await
            .map_err(Error::new)
    }
}
/* implement! {
    ServiceAdapter : RepositoryApi,

    use Postgres for Connection : Atomic;

    User    as UserRepository<Connection>,
    Session as SessionRepository<Connection>,
    OAuth   as OAuthRepository<Connection>;


} */

/* #[async_trait]
impl<Pg, Mg, Connection, MgConn, User, Session, OAuth> RepositoryApi
    for ServiceAdapter<Pg, Mg, Connection, MgConn, User, Session, OAuth>
where
    Self: RepositoryAccess<Connection> + RepositoryAccess<MgConn>,
    Pg: DBConnect<Connection = Connection>,
    Mg: DBConnect<Connection = MgConn>,
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
    Pg: DBConnect<Connection = Connection>,
    Mg: DBConnect<Connection = MgConn>,
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
pub struct ServiceAdapter<Pg, Connection, User, Session, OAuth>
where
    Pg: DBConnect<Connection = Connection>,
{
    #[diesel(Connection)]
    postgres: hextacy::drivers::db::Driver<Pg, Connection>,
    user: std::marker::PhantomData<User>,
    session: std::marker::PhantomData<Session>,
    oauth: std::marker::PhantomData<OAuth>,
} */
