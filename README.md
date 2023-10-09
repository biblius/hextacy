# **⬡ Hextacy ⬡**

A repository designed to bootstrap backend development. Hextacy is a work in progress:

- [x] Database drivers (SQL(diesel, seaorm), Mongo)
- [x] Cache drivers (Redis, TODO: Memcachd)
- [x] Notifications (Email via SMTP)
- [x] Message Queue (Amqp, Redis)
- [ ] Scheduled jobs (crons with tokio-cron)
- [ ] CLI tool for creating app infrastructure (in progress)

## **Feature flags**

```bash
  # Enable everything, sql default is postgres
  - full

  # Enable http, cookie and mime crates
  - web

  # Enable lettre and a simple template SMTP mailer
  - email

  # Enable the specified backend for the specified driver
  - db - postgres|mysql|sqlite - diesel|seaorm
  - db-mongo

  # Enable the redis driver and an in memory cache for quickly prototyping
  - cache-redis
  - cache-inmem
```

## **The server** TODO: This is outdated needs fix pls

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

So far we've been showcasing a simple handler, so let's get to the good stuff.

Notice that we have a `ServiceContract` bound in our handler. Services execute business the business:

- **service.rs**

  ```rust
  pub(super) struct Service<R>
  {
      pub repository: R,
  }

  impl<R> Service<R> where
      R: RepositoryContract,
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

Now we have to define the service component and is when we enter the esoteric realms of rust generics:

- **components.rs**

  ```rust
  use hextacy::drivers::db::{Driver, Driver};
  use std::{marker::PhantomData, sync::Arc};

  #[derive(Debug, Clone)]
  pub struct Repository<D, Conn, User>
  where
      D: Driver<Connection = Conn>,
      User: UserRepository<Conn>,
  {
      driver: Arc<D>,
      user: PhantomData<User>,
  }

  // This one's for convenience
  impl<D, Conn, User> Repository<D, Conn, User>
  where
      D: Driver<Connection = Conn>,
      User: UserRepository<Conn>
  {
      pub fn new(driver: Arc<A>) -> Self {
          Self {
              driver,
              user: PhantomData
          }
      }
  }

  pub trait RepositoryContract {
    fn get_paginated(
        &self,
        page: u16,
        per_page: u16,
        sort: Option<user::SortOptions>,
    );
  }

  impl<D, Conn, User> RepositoryContract for Repository<D, Conn, User>
  where
      D: Driver<Connection = Conn>,
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

`Driver` is a trait used by drivers to establish an actual connection. All concrete clients implement it in their specific ways.

```rust
#[async_trait]
pub trait Driver {
    type Connection;
    async fn connect(&self) -> Result<Self::Connection, DriverError>;
}
```

As you can see, the component's `D` parameter must implement `Driver` with the `Conn` as its connection. Out of the box implementations of drivers exist in the `drivers` module that can satisfy these bounds. This takes care of how we're connecting to the DB.

The `User` bound is simply a bound to a repository the service component will use, which in this case is the `UserRepository`. Since repository methods must take in a connection (in order to preserve transactions) they do not take in `&self`. This is fine, but now the compiler will complain we have unused fields because we are in fact not using them. If we remove the fields, the compiler will complain we have unused trait bounds, so we use phantom data to make the compiler think the struct owns the data.

The `RepositoryContract` serves as a layer of abstraction such that we now do not care what goes in the service's `repository` field so long as it implements `RepositoryContract`. This makes testing the services a breeze since we can now easily mock the contract with `mockall`.

So far we haven't coupled any implementation details to the service, all the service has are calls to some generic drivers, connections and repositories.

This fact is at the core of this architecture and is precisely what makes it so powerful. Not only does this make testing a piece of cake, but it also allows us to switch up our adapters any way we want without ever having to change the business logic. They are completely decoupled.

Do note that the underlying functionality of the repository does not necessarily have to involve a database. The service doesn't care from where the repository obtains its data, it just cares about the signatures. For example, a wrapper around a reqwest client could implement the `Driver` trait with its connection type as the reqwest `Client` struct and could be used to fetch data from an external data source. Neat!

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

To reduce some of the unpleasentness with dealing with so many generics, macros exist to aid the process. If we utilise the `drive!` macro, our `adapter.rs` file becomes a bit more easy on the eyes:

- **_adapter.rs_**

  ```rust
  /* ..imports.. */

  drive! {
    Repository,
    use D for Connection as driver;
    User: UserRepository<Connection>
  }

  #[hextacy::contract]
  impl<D, C, User> Repository<D, C, User>
  where
    C: Send,
    D: Driver<Connection = C> + Send + Sync,
    User: UserRepository<C> + Send + Sync
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
#[hextacy::contract]
impl<D, C, User> Repository<D, C, User>
where
    C: Atomic + Send, // Like thus
    D: Driver<Connection = C> + Send + Sync,
    User: UserRepository<C> + UserRepository<<C as Atomic>::TransactionResult> + Send + Sync,
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
