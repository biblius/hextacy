# **⬡ Hextacy ⬡**

A repository designed to quick start web server development with [actix_web](https://actix.rs/) by providing an extensible infrastructure and a CLI tool to reduce manually writing boilerplate while maintaining best practices.

Hextacy is a work in progress and the api may get absolutely breaking changes.

The kind of project structure this repository uses is heavily based on [hexagonal architecture](<https://en.wikipedia.org/wiki/Hexagonal_architecture_(software)>) also known as the _ports and adapters_ architecture which is very flexible and easily testable. You can read great articles about it [here](https://netflixtechblog.com/ready-for-changes-with-hexagonal-architecture-b315ec967749) and [here](https://blog.phuaxueyong.com/post/2020-05-25-what-architecture-is-netflix-using/).

## **Architecture**

The following is the server architecture intended to be used with the various hextacy helpers, but in order to understand why it is built the way it is, you first need to understand how all the helpers tie together to provide an efficient and flexible architecture.

Backend servers usually, if not always, consists of data stores. _Repositories_ provide methods through which an application's _Adapters_ can interact with to get access to database _Models_.

In this architecture, a repository contains no implementation details. It is simply an interface which adapters utilise for their specific implementations to obtain the underlying model. For this reason, repository methods must always take in a completely generic connection parameter which is made concrete in adapter implementations.

When business level services need access to the database, they can obtain it by having a service adapter struct which is bound to whichever repository traits it needs (to have a better clue what this means, take a look at the server example, or the user example below). For example, an authentication service may need access to a user and session repository.

In the service's definition, its adapter must be constrained by any repository traits the service requires. This will require that the service adapter also takes in generic connection parameters. Since the service should be oblivious to the repository implementation, this means that the driver this service adapter uses to establish database connections must also be generic, since the service cannot know in advance which adapter it will be using.

The generic connection could be mitigated by moving the driver from the business level to the adapter level, but unfortunately we would then lose the ability perfom database transactions (without a nightmare API). The business level must retain its ability to perform atomic queries.

So far, we have 2 generic parameters, the driver and the connection, and we have repositories, interfaces our service repositories can utilise to obtain data, so good!

Because we are now working with completely generic types, we have a completely decoupled architecture (yay), but unfortunately for us, we now have to endure rust's esoteric trait bounds on every service adapter we create (boo). Fortunately for us, we can utilise rust's most excellent feature - macros!

First, let's go step by step to understand why we'll need these macros by examining an example of a simple user endpoint. Check out the [server example](./examples/server/src/) in the examples repo to see how everything is ultimately set up.

## **The server**

First things first, we have to define the data we'll use:

- **data.rs**

  ```rust
  // We expect this in the query params
  // Validify creates a GetUsersPaginatedPayload in the background
  #[derive(Debug, Deserialize, Validify)]
  #[serde(rename_all = "camelCase")]
  pub(super) struct GetUsersPaginated {
    #[validate(range(min = 1, max = 65_535))]
    pub page: Option<u16>,
    #[validate(range(min = 1, max = 65_535))]
    pub per_page: Option<u16>,
  }

  // It must derive Serialize and optionally new for convenience (provided by the
  // derive_new crate)
  #[derive(Debug, Serialize, new)]
  pub(super) struct UserResponse {
    users: Vec<User>,
  }

  impl Response for UserResponse {}
  ```

`GetUsersPaginated` comes in, gets validated, `UserReponse` comes out, simple enough!
We create entry points for the service with handlers:

- **handler.rs**

  ```rust
  use super::{service::ServiceContract, data::GetUsersPaginatedPayload};

  pub(super) async fn get_paginated<S: ServiceContract>(
    data: web::Query<GetUsersPaginatedPayload>,
    service: web::Data<S>,
  ) -> Result<impl Responder, Error> {
      let query = GetUsersPaginated::validify(data.0)?;
      info!("Getting users");
      service.get_paginated(query)
  }
  ```

So far we've been showcasing a simple actix handler, so let's get to the good stuff.

Notice that we have a `ServiceContract` bound in our handler. Services define their api through contract traits:

- **service.rs**

  ```rust
  pub(super) struct Service<R>
  {
      pub repository: R,
  }

  #[hextacy::component]
  impl<R> Service<R> where
      R: RepositoryComponentContract,
  {
      fn get_paginated(&self, data: GetUsersPaginated) -> Result<HttpResponse, Error> {
          let users = self.repository.get_paginated(
              data.page.unwrap_or(1_u16),
              data.per_page.unwrap_or(25),
              data.sort_by,
          )?;

          Ok(UserResponse::new(users)
              .to_response(StatusCode::OK)
              .finish())
      }
  }
  ```

The service has a single field that is completely generic, however in the impl block we bind it to the contract.

The `#[component]` attribute macro will create a `ServiceContract` trait with signatures from the impl block and will implement the trait for the struct. This is done so that the api remains consistent because some components could potentially be swappable. Therefore, the macro should only be used when creating one-of components and is solely here to prevent writing the same items twice and to make the component easily mockable. When using multiple adapters that can be injected into the service, a proper trait should be written out.

The contract provides an api that serves as a layer of abstraction such that we now do not care what goes in the `repository` field so long as it implements `RepositoryContract`. This helps with the generic bounds in the upcoming adapter and makes testing the services a breeze!

Now we have to define our adapter and is when we enter the esoteric realms of rust generics:

- **adapter.rs**

  ```rust
  use hextacy::drivers::db::{Driver, Connect};
  use std::{marker::PhantomData, sync::Arc};

  #[derive(Debug, Clone)]
  pub struct Repository<D, Conn, User>
  where
      D: Connect<Connection = Conn>,
      User: UserRepository<Conn>,
  {
      driver: Driver<D, Conn>,
      user: PhantomData<User>,
  }

  // This one's for convenience
  impl<D, Conn, User> Repository<D, Conn, User>
  where
      D: Connect<Connection = Conn>,
      User: UserRepository<Conn>
  {
      pub fn new(driver: Arc<A>) -> Self {
          Self {
              driver: Driver::new(driver),
              user: PhantomData
          }
      }
  }

  #[hextacy::component]
  impl<D, Conn, User> RepositoryContract for Repository<D, Conn, User>
  where
      D: Connect<Connection = Conn>,
      User: UserRepository<Conn>
  {
    async fn get_paginated(
        &self,
        page: u16,
        per_page: u16,
        sort: Option<user::SortOptions>,
    ) -> Result<Vec<user::User>, Error> {
        let mut conn = self.driver.connect().await?;
        User::get_paginated(&mut conn, page, per_page, sort).await.map_err(Error::new)
    }
  }
  ```

That's a lot of stuff for just fetching users, so let's elaborate.

`Connect` is a trait used by drivers to establish an actual connection. All concrete drivers implement it in their specific ways. It is also implemented by the `Driver` struct. A `Driver` is nothing more than a simple struct:

```rust
// The driver in combination with the Connect trait allows us to fully decouple
// the business logic from the underlying data source implementations
struct Driver<A, C>
where
    A: Connect<Connection = C>,
{
    pub inner: Arc<A>,
}

#[async_trait]
pub trait Connect {
    type Connection;
    async fn connect(&self) -> Result<Self::Connection, DriverError>;
}
```

As you can see, the component's `D` parameter must implement `Connect` with the `Conn` as its connection. Out of the box implementations of drivers exist in the `drivers` module that can satisfy these bounds, but . This takes care of how we're connecting to the DB.

The `User` bound is simply a bound to a repository the service component will use, which in this case is the `UserRepository`. Since repository methods must take in a connection (in order to preserve transactions) they do not take in `&self`. This is fine, but now the compiler will complain we have unused fields because we are in fact not using them. If we remove the fields, the compiler will complain we have unused trait bounds, so we use phantom data to make the compiler think the struct owns the data.

So far we haven't coupled any implementation details to the service, all the service has are calls to some generic drivers, connections and repositories.

This fact is at the core of this architecture and is precisely what makes it so powerful. Not only does this make testing a piece of cake, but it also allows us to switch up our adapters any way we want without ever having to change the business logic. They are completely decoupled.

Do note that the underlying functionality of the repository does not necessarily have to involve a database. The service doesn't care from where the repository obtains its data, it just cares about the signatures. For example, a wrapper around a reqwest client could implement the `Connect` trait with its connection type as the reqwest `Client` struct and could be used to fetch data from an external data source. Neat!

Finally, we'll concretise everything in the setup:

- **setup.rs**

  ```rust
  pub(crate) fn routes(pg: Arc<Postgres>, rd: Arc<Redis>, cfg: &mut web::ServiceConfig) {
    let service = UserService {
        repository: Repository::<Postgres, DieselConnection, PgUserAdapter>::new(pg.clone()),
    };
    let auth_guard = interceptor::AuthGuard::new(pg, rd, Role::User);

    cfg.app_data(Data::new(service));

    // Show all
    cfg.service(
        web::resource("/users")
            .route(web::get().to(handler::get_paginated::<
                UserService<Repository<Postgres, DieselConnection, PgUserAdapter>>,
            >))
            .wrap(auth_guard),
    );
  }
  ```

I'll admit it, the trait bounds do look kind of ugly, but seeing as this is the only place where we concretise our types, we never have to worry about the rest of the service breaking when we makes changes in our adapters. The concrete repository can be extracted to a type definition to reduce the amount of places where it needs to be changed and for visibility.

To reduce some of the unpleasentness with dealing with so many generics, macros exist to aid the process. If we utilise the `adapt!` macro, our `adapter.rs` file becomes a bit more easy on the eyes:

- **_adapter.rs_**

  ```rust
  /* ..imports.. */

  adapt! {
    Repository,
    use D for Connection as driver;
    User: UserRepository<Connection>
  }

  #[hextacy::component]
  impl<D, Connection, User> Repository<D, Connection, User> 
  where
    Connection: Send,
    D: Connect<Connection = Connection> + Send + Sync,
    User: UserRepository<Connection> + Send + Sync
  {
      async fn get_paginated(
          &self,
          page: u16,
          per_page: u16,
          sort: Option<user::SortOptions>,
      ) -> Result<Vec<user::User>, Error> {
          let mut conn = self.driver.connect().await?;
          User::get_paginated(&mut conn, page, per_page, sort).await.map_err(Error::new)
      }
  }
  ```

Looks much better! You can read more about how the macro works in the `hextacy::db` module.

### **Transactions**

The reason for repositories always taking in a connection in their methods is transactions. Since business level services should have the ability to rollback transactions if anything goes south, we have to somehow enable their adapters to suport transactions.

Transactions could theoretically be started in the business level, but I prefer to group complicated repository logic to a single adapter call that takes care of everything. This way we never have to pass in connections to the service component's methods, but if there is some complex logic in the business layer that has to affect the outcomes of transactions, its api can be defined in a way that lets us pass in connections/transactions to it so we remain flexible.

The `Atomic` trait provides an interface for any repository to start, commit or rollback a transaction by binding the generic connection used in the repository to the `Atomic` trait. This bound can be introduced in the API implementation for the service adapter:

```rust
#[hextacy::component]
impl<D, Connection, User> RepositoryContract for Repository<D, Connection, User>
where
    D: Connect<Connection = Connection> + Send + Sync,
    User: UserRepository<Connection> + UserRepository<<Connection as Atomic>::TransactionResult> + Send + Sync,
    Connection: Atomic + Send, // Like thus
{
  async fn get_paginated(
      &self,
      page: u16,
      per_page: u16,
      sort: Option<user::SortOptions>,
  ) -> Result<Vec<user::User>, Error> {
      let conn = self.driver.connect().await?;
      let mut tx = conn.start_transaction().await?; // Provided by the Atomic trait
      match User::get_paginated(&mut tx, page, per_page, sort).await {
        Ok(user) => {
          <Connection as Atomic>::commit_transaction(tx).await?;
          Ok(user)
        },
        Err(e) => {
          <Connection as Atomic>::abort_transaction(tx).await?;
          Err(e.into())
        }
      }
  }
}
```

Atomic is implemented for all out of the box driver connections in hextacy. The reason why it looks the way it does is to provide a uniform API for transactions that are done on connections and transactions that return a transaction struct.

For example, diesel uses a transaction manager which starts the transaction on the connection and returns a `Result<()>` while seaorm's transaction manager returns a `Result<DatabaseTransaction>`. If we were to directly implement these it would break our API, since different code needs to be executed depending on the driver (in seaorm we wouldn't just be able to pass the connection to our repository calls since the transaction is located in the struct which must be used in order to tell the ORM to perform the operations atomically).

The `Atomic` trait normalises the API; For diesel we simply return the connection in `start_transaction` and use that, while for seaorm we return the `DatabaseTransaction`.

The API is normalised because anything that's returned is in `Atomic::TransactionResult`. If you take a look at the above code block, you'll notice we've bound `User` to a repository that now must operate on both the connection and its transaction result.

For connection based transactions (like diesel and mongo), the `Atomic::TransactionResult` will be the very same connection, meaning we do not have to create an additional implementation for the transaction. In seaorm however, we need to create an implementation for the transaction as well. Usually ORMs provide a trait that represents a connection, so we can just implement the repository with it.

To elaborate further, here's what a repository would look like:

- **repository/user.rs**

```rust
pub trait UserRepository<C> {
    fn get_paginated(
        conn: &mut C,
        page: u16,
        per_page: u16,
        sort_by: Option<SortOptions>,
    ) -> Result<Vec<User>, RepoAdapterError>;
}
```

The adapter just implements the `UserRepository` trait and returns the model using its specific ORM. This concludes the architectural part (for now... :).

## **hextacy**

Feature flags:

```bash
  - full - Enables all the feature below

  - db - Enables mongo, diesel, seaorm and redis
  - ws - Enable the WS session adapter and message broker

  - postgres-diesel - Enables the diesel postgres driver
  - postgres-seaorm - Enables the seaorm postgres driver
  - mongo - Enables the mongodb driver
  - redis - Enables the redis driver and cache access trait
  - email - Enables the SMTP driver and lettre
```

- ### **db**

  Contains a collection of traits to implement on structures that access databases and interact with repositories. Provides macros to easily generate repository structs as shown in the example.

- ### **drivers**

  Contains structures implementing driver specific behaviour such as connecting to and establishing connection pools with database, cache, smtp and http servers. All the connections made here are generally shared throughout the app with Arcs. Check out the [drivers readme](./hextacy/src/drivers/README.md)

- ### **logger**

  The `logger` module utilizes the [tracing](https://docs.rs/tracing/latest/tracing/), [env_logger](https://docs.rs/env_logger/latest/env_logger/) and [log4rs](https://docs.rs/log4rs/latest/log4rs/) crates to setup logging either to stdout or a `server.log` file, whichever suits your needs better.

- ### **crypto**

  Contains cryptographic utilities for encrypting and signing data and generating tokens.

- ### **web**

  Contains various helpers and utilities for HTTP and websockets.

  - **http**

    The most notable here are the _Default security headers_ middleware for HTTP (sets all the recommended security headers for each request as described [here](https://www.npmjs.com/package/helmet)) and the _Response_ trait, a utility trait that can be implemented by any struct that needs to be turned in to an HTTP response. Also some cookie helpers.

  - **ws**

    Module containing a Websocket session handler.

    Every message sent to this handler must have a top level `"domain"` field. Domains are completely arbitrary and are used to tell the ws session which datatype to broadcast.

    Domains are internally mapped to data types. Actors can subscribe via the broker to specific data types they are interested in and WS session actors will in turn publish them whenever they receive any from their respective clients.

    Registered data types are usually enums which are then matched in handlers of the receiving actors. Enums should always be untagged, so as to mitigate unnecessary nestings from the client sockets.

    Uses an implementation of a broker utilising the [actix framework](https://actix.rs/book/actix/sec-2-actor.html), a very cool message based communication system based on the [Actor model](https://en.wikipedia.org/wiki/Actor_model).

    Check out the `web::ws` module for more info and an example of how it works.

- ### **cache**

  Contains a cacher trait which can be implemented for services that require access to the cache. Each service must have its cache domain and identifiers for cache seperation. The `AuthCacheAccess` and `KeyPrefix` traits can be used for such purposes.
