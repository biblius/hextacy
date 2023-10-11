# Infrastructure

So far we have only been dealing with behaviour, now it's time to implement that behaviour on concrete units. There are 2 main pieces of infrastructure our application is missing; The interaction and the plumbing.

Since we all know what an HTTP controller is, we'll be creating one with `axum` for the interaction. We choose HTTP because most people are familiar with it and it's the simplest to set up, though we could've chosen anything because nothing in the service specifies how it should be interacted with. Honourable mentions include a desktop or CLI app.

For the plumbing, i.e. database and queue implementations, we'll be using postgres and redis with pubsub. We'll use them because, again, they are familiar to most people, but we could've chosen anything so long it can be plugged in as a `Driver` with its `Atomic` connection, and the publisher satisfies the `Producer` trait.

## Adapters

Since we'll be using postgres, generally we need to do the following:

1. Create migrations that will define our `users` and `sessions` tables, run them
2. Scan our schema with an ORM, in our case sea-orm (optional)
3. Create ORM entities that correspond to our SQL data

We won't go over these steps because they depend on the implementation, you may or may not use an ORM depending on preference. In any case, the first step is always performed. Since we'll be using sea-orm, we perform step 2, and subsequently sea-orm will generate the necessary ORM entities, completing step 3. All we need to do now is write the `From` implementations for our application models. The ORM entities allow us to perform queries on their respective tables.

For more detail see [migr](https://github.com/biblius/migr), a very simple tool for generating migrations, [how to generate entities with sea-orm](https://www.sea-ql.org/sea-orm-tutorial/ch01-04-entity-generation.html), and the [examples directory](https://github.com/biblius/hextacy/tree/master/examples/template/src/db).

Now that we have the necessary entities to perform database queries, we can create our adapter. For brevity, we'll be showcasing only the `UserAdapter` here, the session adapter can be viewed in the examples.

```rust
#[derive(Debug, Clone)]
pub struct UserAdapter;

#[async_trait]
impl<C> UserRepository<C> for UserAdapter
where
    C: ConnectionTrait + Send + Sync,
{
    async fn get_by_id(&self, conn: &mut C, id: Uuid) -> Result<Option<User>, AdapterError> {
        UserEntity::find_by_id(id)
            .one(conn)
            .await
            .map_err(AdapterError::SeaORM)
            .map(|u| u.map(User::from))
    }

    async fn get_by_username(
        &self,
        conn: &mut C,
        username: &str,
    ) -> Result<Option<User>, AdapterError> {
        UserEntity::find()
            .filter(Column::Username.eq(username))
            .one(conn)
            .await
            .map_err(AdapterError::SeaORM)
            .map(|user| user.map(User::from))
    }

    async fn create(
        &self,
        conn: &mut C,
        username: &str,
        password: &str,
    ) -> Result<User, AdapterError> {
        let user: UserModel = User::new(username.to_string(), password.to_string()).into();
        UserEntity::insert(user)
            .exec_with_returning(conn)
            .await
            .map(User::from)
            .map_err(AdapterError::SeaORM)
    }
}
```

Ah, finally we see some action! The code is pretty self-explanatory so we won't go over it in too much detail.

`ConnectionTrait` is the sea-orm specific trait which can be passed into the `exec` calls on entities. This trait is implemented directly on a `sea_orm::DatabaseConnection` and `sea_orm::DatabaseTransaction`. Fortunately, most ORMs provide a connection trait so we don't have to implement the adapters for both their connection and transaction - that would be painful. We can obtain a `C: ConnectionTrait` via the sea-orm driver - a thin wrapper around a sea-orm connection pool that implemments `Driver`, making it suitable for our service.

_Quick sidenote:_

_There are ORMs that start transactions in place on connections. These implement `Atomic` by starting the transaction and then just returning the connection. The reason `Atomic` exists is because of these different implementations, we need a way to abstract away the specific way a transaction is started, we do so with the `Atomic::TransactionResult`._

One small thing to note is that we want to keep UUID generation within our control. Giving control to the database would mean the most critical part of our model is out of the application's control which would introduce problems later down the line if we ever need to switch our adapters. Here we're handling the ID generation in the user's `new` function.

For the publisher, we can use hextacy's [RedisPublisher](https://github.com/biblius/hextacy/blob/master/hextacy/src/adapters/queue/redis.rs). It has the ability to create a producer for any given message as long as it implements `Serialize`. It implements the `Producer` trait which is just what we need.

Since we will at some point have to make a concrete instance of our service, to reduce the boilerplate of specifying every one of its components wherever we use it, we create a type alias:

```rust
pub type AuthenticationService = Authentication<
    SeaormDriver,
    UserAdapter,
    SessionAdapter,
    RedisPublisher,
>;
```

Now instead of specifying (and inevitably changing) the adapters everywhere we want to use the service, we have a single centralised location where we define its configuration and use this type wherever we want to use the service.

We'll figure out how we manage the necessary state for it in a bit because first we'll define the controllers.

## Controllers

In this part we'll hook up a single handler function to a service because they look the same for each, give or take a cookie/header.

We now define the HTTP handler for the service's `login` function.

```rust
#[derive(Debug, Deserialize, Validify)]
pub struct Login {
    #[validate(length(min = 1))]
    pub username: String,
    #[validate(length(min = 1))]
    pub password: String,
    pub remember: bool,
}

pub async fn login(
    State(service): State<AuthenticationService>,
    Json(data): Json<LoginPayload>,
) -> Result<Response<String>, Error> {
    let Login {
        username,
        password,
        remember,
    } = Login::validify(data).map_err(Error::new)?;
    let session = service.login(&username, &password, remember).await?;
    let session_id = session.id.to_string();
    let cookie = session_cookie("S_ID", &session_id, false);
    MessageResponse::new("Successfully logged in")
        .into_response(StatusCode::OK)
        .with_cookies(&[cookie])?
        .json()
        .map_err(Error::new)
}

// Helper for creating a cookie
pub fn session_cookie<'a>(
    key: &'a str,
    value: &'a str,
    expire: bool,
) -> Cookie<'a> {
    CookieBuilder::new(key, value)
        .path("/")
        .domain("mysupercoolsite.com")
        .max_age(if expire { Duration::ZERO } else { Duration::days(1) })
        .same_site(SameSite::Lax)
        .http_only(true)
        .secure(true)
        .finish()
}
```

The first thing we do is define the data object we intend to accept from the client. The `Validify` derive macro exposes a `validify` method for the struct. It also creates a payload struct which we use in the `login` handler. The first argument to this function is the service type we've defined earlier, wrapped in an `axum::extractor::State`. Through it we obtain a reference to the concrete service.

In the next section, we'll see how we can manage state and hook everything up so we have a working application.
