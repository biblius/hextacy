# Core

The next few sections of the booklet will provide some examples on how to model a decoupled application and how hextacy can be utilised to efficiently write application code while hiding away rust's unavoidable boilerplate.

## Requirements

Let's imagine we are tasked with creating an authentication service. We choose an auth service because it is simple enough for everyone to understand while still being able to highlight the importance of a layered architecture. For brevity's sake, we will keep the service very simple and we will not provide a `logout` method for user retention. After an intense brainstorming session we have determined the following:

The service must:

- expose 2 methods: `register` and `login`.

- be able to work with 2 models (entities): `User` and `Session`.

- notify any interested third parties a user registered via a message broker.

## Implementation

For brevity, we will not be writing out the application plumbing (imports, errors, etc.) because we want to focus solely on the design. Full examples with plumbing can be viewed [in the examples directory](https://github.com/biblius/hextacy/tree/master/examples/).

### Models (Entities)

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

_ORM entities are distinct (and confusingly named the same way) from our application entities, which from now on we will refer to as application models. An entity is a concept from domain driven design representing a data structure with semantic meaning to our application. Since we are dealing with authentication, the `User` and `Session` structs are the application entities as they represent core concepts from the real world. Each entity (application model) must be uniquely identifiable - as such, the ID generation for those entities must be in the hands of our app, rather than the underlying persistence implementation._

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
    ) -> Result<Option<Session>, AdapterError>;

    async fn create(
        &self,
        conn: &mut C,
        user: &User,
        expires: bool,
    ) -> Result<Session, AdapterError>;
}
```

The service will now be able utilise these definitions and in doing so won't be coupled to any particular implementation. If you're wondering why the `C`, we could theoretically design a repository with no generics, but it will introduce problems later down the line when we stray off the happy path.

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
    ) -> AppResult<Session> {
        let mut conn = self.repo.connect().await?;

        let user = match self.user_repo.get_by_username(&mut conn, username).await {
            Ok(Some(user)) => user,
            Ok(None) => return Err(AuthenticationError::InvalidCredentials.into()),
            Err(e) => return Err(e.into()),
        };

        let valid = hextacy::crypto::bcrypt_verify(password, &user.password)?;
        if !valid {
            return Err(AuthenticationError::InvalidCredentials.into());
        }

        let session = self
            .session_repo
            .create(&mut conn, &user, !remember)
            .await?;

        Ok(session)
    }
}
```

_In the first circle of generics hell we can observe the famous Send and Sync bounds from the async rust habitat..._

In the impl block's definition, we introduced the necessary generics for the service and we've bound those generics to the traits we want the service to use. We are essentially saying to the compiler "_The authentication struct can use the `login` method if and only if its `driver` field implements `Driver` and its `*_repo` fields can work on the connection obtained from that driver_".

The [Driver](../Driver.md) trait is a completely generic trait that exposes one method - `connect`. It is literally just

```rust
#[async_trait]
pub trait Driver {
    type Connection;
    async fn connect(&self) -> Result<Self::Connection, DriverError>;
}
```

We need this trait because we've defined our repository to take in a generic `C` and now we can obtain that `C` from the driver. We still don't know which connection that will be - this is the whole point of the `Driver` trait and is how our service still remains oblivious to the adapter it will use.

Because the generics are bound to repositories we get access to the necessary repository methods and can get a hold of our application models. So far, no implementation details are exposed to the service. The only thing the service is aware of is that it can create some connection and use that connection for its repositories.

The real beauty of using a driver is in the next step, when we define our `register` method.

```rust
// Same impl block as for the `login` method
async fn register(&self, username: &str, password: &str) -> AppResult<Session> {
    let mut conn = self.driver.connect().await?;

    match self.user_repo.get_by_username(&mut conn, username).await {
        Ok(None) => {}
        Ok(Some(_)) => return Err(AuthenticationError::UsernameTaken.into()),
        Err(e) => return Err(e.into()),
    };

    let hashed = hextacy::crypto::bcrypt_hash(password, 10)?;

    let user = self.user_repo.create(&mut conn, username, &hashed).await?;
    let session = self.session_repo.create(&mut conn, &user, true).await?;

    Ok(session)
}

```

_...but this just looks like the login method, what's up?_

We now stray from the happy path.

### Transactions

Imagine the above `session_repo.create` call failed and the function returned an error. A user would still be created, but they would receive no session and they wouldn't be granted application access.
This might not be a big deal for our simple auth service since the user could just login and continue on with their life, but imagine things are not so simple.

Imagine we have to execute multiple state changes to multiple repositories. When there are multiple pending state changes, we want to persist those changes only if all of them succeed, and conversely we want to revert all changes if any of them fail. For this we need transactions. In order to use transactions, we must devise a way for our driver, specifically its connection, to allow us to perform atomic queries with it. Most connections/db clients provide this out of the box with 3 simple methods:

- `start_transaction`
- `commit_transaction`
- `rollback_transaction`

For this purpose, hextacy provides this functionality on any generic connection via the [Atomic](../Driver.md#Atomic) trait.
Because transactions usually operate on the same connections, i.e. queries on a connection that started a transaction will all be executed within that transaction's context, we get the answer to the age old question of "Why put the `C` in the repository?".

If our repository methods did not take in a `C`, then we would not be able to pass a transaction through multiple repository calls.

We now update the register method to support transactions and isolate the creation of users and sessions to a neat little function. `//` marks lines added/changed.

```rust
use hextacy::{Atomic, Driver};

#[async_trait]
impl<D, UR, SR> Authentication<D, UR, SR>
where
  D: Driver + Send + Sync,
  D::Connection: Atomic + Send, //
  UR:
    UserRepository<D::Connection> +
    UserRepository<<D::Connection as Atomic>::TransactionResult> + //
    Send +
    Sync,
  SR:
    SessionRepository<D::Connection> +
    SessionRepository<<D::Connection as Atomic>::TransactionResult> + //
    Send +
    Sync,
{
    pub async fn register(&self, username: &str, password: &str) -> AppResult<Session> {
        let mut conn = self.driver.connect().await?;

        match self.user_repo.get_by_username(&mut conn, username).await {
            Ok(None) => {}
            Ok(Some(_)) => return Err(AuthenticationError::UsernameTaken.into()),
            Err(e) => return Err(e.into()),
        };

        let hashed = hextacy::crypto::bcrypt_hash(password, 10)?;

        let mut tx = conn.start_transaction().await?;
        match self //
            .create_user_and_session(&mut tx, username, &hashed)
            .await
        {
            Ok(session) => {
                <Repo::Connection as Atomic>::commit_transaction(tx).await?;
                Ok(session)
            }
            Err(e) => {
                <Repo::Connection as Atomic>::abort_transaction(tx).await?;
                Err(e)
            }
        }
    }

    pub async fn create_user_and_session( //
        &self,
        tx: &mut <Repo::Connection as Atomic>::TransactionResult,
        username: &str,
        password: &str,
    ) -> AppResult<Session> {
        let user = self.user_repo.create(tx, username, password).await?;
        let session = self.session_repo.create(tx, &user, true).await?;
        Ok(session)
    }
}
```

_...and in the 9th circle of generics hell we can observe the impenetrable wall of ultimate bounds_

I know, I know - who in their right mind would want to write all of this out? Our service has only 2 repositories and already half of our file is noisy generics. While we are reaping the benefit of having atomic queries we've stumbled upon another problem - boilerplate. We'll figure that one out in the next section, but first let's focus on how the code differs from our original implementation.

Now, before we start with the state changes in our database we start a transaction. This is possible because we've bound the driver's connection to `Atomic`. When we get the results of `create_user_and_session`, we make sure to perform the necessary action on the transaction, ensuring the changes are only committed if everything was successful. This is where rust absolutely shines because we have total control on each of our interactions.

One other thing to note for this approach is encapsulation. Since now the service is responsible for obtaining connections, one could argue that the driver does not belong in the service implementation logic since it is doing what is supposedly the repository's job. Repositories can be designed with no generics, as stated previously, and this would allow the service to completely remove the driver from its definition. This is a completely valid decision if one does not need atomicity in their queries and makes defining services with `Box<dyn Repository>` a great option. On the other hand, when we need transactions, the service always has the necessary context to reason about whether or not a transaction should succeed and should be left up to the service, in which case the `C` is unavoidable.

In the next section we'll tear down the wall of generics and streamline the process of writing services using hextacy.
