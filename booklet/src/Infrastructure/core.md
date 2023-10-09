# Core

The next few sections of the booklet will provide some examples on how to model a decoupled application and how hextacy can be utilised to efficiently write application code while hiding away rust's unavoidable boilerplate.

## Requirements

Let's imagine we are tasked with creating an authentication service. For brevity's sake, we will keep the service very simple and we will not provide a `logout` method. After an intense brainstorming session we have determined the following:

The service must:

- expose 2 methods: `register` and `login`.

- be able to work with 2 models (entities): `User` and `Session`.

- notify any interested third parties that a user registered via a message broker.

## Implementation

For brevity, we will not be writing out the application plumbing (imports, errors, etc.) because we want to focus solely on the design. That said, full examples with plumbing can be viewed [here](https://github.com/biblius/hextacy/tree/master/examples/).

### Models

First things first, we have to define the application models:

```rust
pub struct User {
    id: Uuid,
    username: String,
    password: String,
    created_at: NaiveDateTime, // from chrono
}

pub struct Session {
    id: Uuid,
    user_id: Uuid,
    created_at: NaiveDateTime,
    expires_at: NaiveDateTime,
}
```

These models must be kept separate from ORM-specific entities. Any entity obtained from an ORM must be convertable to its respective application model. Here the `From` trait is our friend, but we will omit the implementation as it is straightforward.

### Repository

We now define a set of interactions with a persistence layer. You can think of repositories as contracts an adapter must fulfill for it to be injected into a service.

```rust
#[async_trait]
pub trait UserRepository<C> {
    async fn get_by_username(
        &self,
        conn: &mut C,
        username: &str,
    ) -> Result<Option<User>, AdapterError>;

    async fn create(
        &self,
        conn: &mut C,
        username: &str,
        password: &str,
    ) -> Result<User, AdapterError>;
}

#[async_trait]
pub trait SessionRepository<C> {
    async fn get_valid_by_id(
        &self,
        conn: &mut C,
        id: Uuid,
        csrf: Uuid,
    ) -> Result<Option<Session>, AdapterError>;

    async fn create(
        &self,
        conn: &mut C,
        user: &User,
        expires: bool,
    ) -> Result<Session, AdapterError>;
}
```

You might be wondering why the generic `C` bound. It will all become clear later, but for now you should just know that it enables us to easily perform transactions. We could theoretically design a repository with no generics, but it will introduce problems later down the line when we have the need for atomicity.

### Service

We now define the core authentication service struct. For the time being we will disregard the message broker requirement and focus solely on the first 2.

```rust
pub struct Authentication<D, UR, SR> {
    driver: D,
    user_repo: UR,
    session_repo: SR,
}
```

Since we do not know which adapters the service will be instantiated with, we must define it in terms of generics. Another option would be to define the `*_repo` fields using trait objects, i.e. `Box<dyn UserRepository<C>>`, but then we would have to introduce another generic for the connection, namely `C`, which arguably does not help us when we enter generics hell in the next step when defining the core functionality.

We now define the `login` method.

```rust
use hextacy::Driver;

#[async_trait]
impl<D, UR, SR> Authentication<D, UR, SR>
where
  D: Driver + Send + Sync,
  D::Connection: Send,
  UR: UserRepository<Driver::Connection> + Send + Sync,
  SR: SessionRepository<Driver::Connection> + Send + Sync,
{
    async fn login(
        &self,
        username: &str,
        password: &str,
        remember: bool,
    ) -> AppResult<ClientSession> {
        let mut conn = self.repo.connect().await?;

        let user = match self.user_repo.get_by_username(&mut conn, username).await {
            Ok(Some(user)) => user,
            Ok(None) => return Err(AuthenticationError::InvalidCredentials.into()),
            Err(e) => return Err(e.into()),
        };

        let hashed = hextacy::crypto::bcrypt_hash(password, 10)?;
        if hashed != password {
            return Err(AuthenticationError::InvalidCredentials.into());
        }

        let session = self
            .session_repo
            .create(&mut conn, &user, !remember)
            .await?;

        Ok(session.into())
    }
}
```

No wait, don't go, I promise it'll all make sense!

Our impl block is quite a mouthful so let's break it down.

In the impl block's definition, we introduced the necessary generics for the service and we've bound those generics to the traits we want the service to use.

We are essentially saying to the compiler "_The authentication struct can use the `login` method if and only if its `driver` field implements `Driver` and its `*_repo` fields can work on the connection obtained from that driver_".

The [Driver](../Driver.md) trait is a completely generic trait that exposes one method - `connect`. We need this trait because we've defined our repository to take in a generic `C` and now we can obtain that `C` from the driver. We still don't know which connection that will be - this is the whole point of the `Driver` trait and is how our service still remains oblivious to the adapter it will use.

Now, because the generics are bound to repositories we get access to the necessary repository methods and can get a hold of our application models. So far, no implementation details are exposed to the service. The only thing the service is aware of is that it can create some connection and use that connection for its repositories.

The real beauty of using a driver is in the next step, when we define our `register` method.

```rust
// Same impl block as for the `login` method
async fn register(&self, username: &str, password: &str) -> AppResult<ClientSession> {
    let mut conn = self.repo.connect().await?;

    match self.user_repo.get_by_username(&mut conn, username).await {
        Ok(None) => {}
        Ok(Some(_)) => return Err(AuthenticationError::UsernameTaken.into()),
        Err(e) => return Err(e.into()),
    };

    let hashed = hextacy::crypto::bcrypt_hash(password, 10)?;

    let user = self.user_repo.create(&mut conn, username, &hashed).await?;
    let session = self.session_repo.create(&mut conn, &user, true).await?;

    Ok(session.into())
}

```

Ok, so you might be wondering where is this beauty we've mentioned - this just looks the same as the login method, big deal!

You would be completely right to wonder this and the above implementation does not in fact differ from the login method (other than the fact it executes different code). The beauty comes when we introduce database transactions.

### Transactions

Imagine the above `session_repo.create` call failed and the function returned an error. A user would still be created, but they would receive no session and they wouldn't be granted application access.

This might not be a big deal for our simple auth service since the user could just login and continue on with their life, but imagine things are not so simple.

Imagine we have to execute multiple state changes to multiple repositories. If you've ever had problems with this in the real world, you already know that having incomplete state/partial updates in a database is when being a vegetable farmer doesn't start to look all that bad.

Luckily, smarter people than we have thought about this and they have provided us with a simple, yet effective tool to mitigate this problem - transactions! If you don't know what those are, they are essentially the database equivalent of all or nothing.

In order to get transactional integrity for our queries, we must devise a way for our driver, specifically its connection, to
perform atomic queries. Most connections/db clients provide this out of the box with 3 simple methods:

- `start_transaction`
- `commit_transaction`
- `rollback_transaction`

For this purpose, hextacy provides this functionality on any generic connection via the [Atomic](../Driver.md#Atomic) trait.
Because transactions usually operate on the same connections, i.e. queries on a connection that started a transaction will all be executed within that transaction's context, we get the answer to the age old question of "Why put the `C` in the repository?".

If our repository methods did not take in a `C`, then we would not be able to pass a transaction through multiple repository calls.
