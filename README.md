# **⬡ Hextacy ⬡**

A repository designed to quick start web server development with [actix_web](https://actix.rs/) by providing an extensible infrastructure and a CLI tool to reduce manually writing boilerplate while maintaining best practices.

The kind of project structure this repository uses is heavily based on [hexagonal architecture](https://en.wikipedia.org/wiki/Hexagonal_architecture_(software)) also known as the *ports and adapters* architecture which is very flexible and easily testable. You can read great articles about it [here](https://netflixtechblog.com/ready-for-changes-with-hexagonal-architecture-b315ec967749) and [here](https://blog.phuaxueyong.com/post/2020-05-25-what-architecture-is-netflix-using/).

## **Get started**

1. Create a real `.env` file filling out the example file with your desired parameters and create the database you entered in the file. You do not have to fill out fields you don't intend to use.

    - For the Email part, use [this spec](https://support.google.com/mail/answer/7126229?hl=en#zippy=%2Cstep-change-smtp-other-settings-in-your-email-client) to set up the SMTP host(smtp.gmail.com) and port(465) and [follow these instructions](https://support.google.com/accounts/answer/185833?hl=en#zippy=%2Cwhy-you-may-need-an-app-password) to generate an app password. The password can then be used for the `SMTP_PASSWORD` variable. For the sender and username enter your email address.

2. Install the CLI tool via

    ```bash
    cargo install --path xtc
    ```

3. For the secrets, (namely REG_TOKEN_SECRET and COOKIE_SECRET) run

    ```bash
    xtc c secret <SECRET_NAME> [-n <NAME>] [-l <LENGTH>]
    ```

    and replace the example ones with them.

    Run migrations via

    ```bash
    xtc m run
    ```

    Give yourself a pat on the back since you've made it this far and optionally check out all the commands with `xtc -h`.

    Run the server with

    ```bash
    cargo run -p server
    ```

    and load the postman collection located in `misc` to interact with it.

You now have a ready to go infrastructure, now go make that billion $$$ app!

## **Architecture**

The following is the server architecture intended to be used with the various xtc helpers, but in order to understand why it is built the way it is, you first need to understand how all the helpers tie together to provide an efficient and flexible architecture.

Backend server development usually, if not always, consists of data stores. *Repositories* provide methods through which the application's *Adapters* can interact with to get access to database *Models*.

In this architecture, a repository contains no implementation details. It is simply an interface which adapters utilise for their specific implementations to obtain the underlying model. For this reason, repository methods must always take in a completely generic connection parameter. This generic parameter is made concrete in adapter implementations.

When business level services need access to the database, they can obtain it by having a repository struct which is bound to whichever repository traits it needs (to have a better clue what this means, take a look at the server example, or the user example below). For example, an authentication service may need access to a user and session repository.

In the service's definition, its repository must be constrained by any repository traits the service requires. This will require that the intermediate service repository also takes in generic connection parameters. Since the service should be oblivious to the repository implementation, this means that the client this intermediate repository uses to establish database connections must also be generic, since the service cannot know in advance which adapter it will be using.

The generic connection could be mitigated by moving the client from the business level to the adapter level, but unfortunately we would then lose the ability perfom database transactions (without a nightmare API). The business level must retain its ability to perform atomic queries.

So far, we have 2 generic parameters, the client and the connection, and we have repositories, interfaces our service repositories can utilise to obtain data, so good!

Because we are now working with completely generic types, we have a completely decoupled architecture (yay), but unfortunately for us, we now have to endure rust's esoteric trait bounds on every intermediate repository we create (boo). Fortunately for us, we can utilise rust's most excellent feature - macros!

First, let's go step by step to understand why we'll need these macros by examining an example of a simple user endpoint:

## **The server**

  First things first, we have to define the data we'll use:
  
- **data.rs**
  
    ```rust
    // We expect this in the query params
    // Validify creates a GetUsersPaginatedPayload in the background
    #[derive(Debug, Deserialize)]
    #[validify]
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
    use super::{contract::ServiceContract, data::GetUsersPaginatedPayload};
  
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

Notice that we have a `ServiceContract` bound in our handler. Services define their api through contracts. Contracts are nothing more than traits (interfaces) through which we interact with the service:

- **contract.rs**

  ```rust
  pub(super) trait ServiceContract {
    fn get_paginated(&self, data: GetUsersPaginated) -> Result<HttpResponse, Error>;
  }

  pub(super) trait RepositoryContract {
      fn get_paginated(
          &self,
          page: u16,
          per_page: u16,
          sort: Option<user::SortOptions>,
      ) -> Result<Vec<User>, Error>;
  }
  ```

These contracts define the behaviour we want from our service and the infrastructure it will use.
The service contract is implemented by the service struct:

- **service.rs**

  ```rust
  pub(super) struct UserService<R>
  where
      R: RepositoryContract,
  {
      pub repository: R,
  }

  impl<R> ServiceContract for UserService<R>
  where
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

The service has a single field that must implement the contract. This contract serves as a layer of abstraction such that we now do not care what goes in the `repository` field so long as it implements `RepositoryContract`. This helps with the generic bounds in the upcoming repository and makes testing the services a breeze!

Now we have to define our repository and is when we enter the esoteric realms of rust trait bounds:

- **adapter.rs**

  ```rust
  use hextacy_derive::Repository;
  use hextacy::clients::db::{Client, DBConnect};
  use std::{marker::PhantomData, sync::Arc};

  #[derive(Debug, Clone, Repository)]
  #[postgres(Conn)]
  pub struct Repository<C, Conn, User>
  where
      C: DBConnect<Connection = Conn>,
      User: UserRepository<Conn>,
  {
      postgres: Client<C, Conn>,
      user: PhantomData<User>,
  }

  // This one's for convenience
  impl<C, Conn, User> Repository<C, Conn, User>
  where
      C: DBConnect<Connection = Conn>,
      User: UserRepository<Conn>
  {
      pub fn new(client: Arc<A>) -> Self {
          Self {
              client: Client::new(client),
              user: PhantomData
          }
      }
  }

  impl<C, Conn, User> RepositoryContract for Repository<C, Conn, User> 
  where
      Self: RepositoryAccess<Conn>,
      C: DBConnect<Connection = Conn>,
      User: UserRepository<Conn>
  {
    fn get_paginated(
        &self,
        page: u16,
        per_page: u16,
        sort: Option<user::SortOptions>,
    ) -> Result<Vec<user::User>, Error> {
        let mut conn = self.connect()?;
        User::get_paginated(&mut conn, page, per_page, sort).map_err(Error::new)
    }
  }
  ```

That's a lot of stuff for just getting users out of the database.

The `Repository` derive implements the `RepositoryAccess` trait using `PgPoolConnection` as its connection type, since we annotated it with
postgres.

`RepositoryAccess` is a simple trait that is generic over the connection and gives its implementors a `connect()` method to establish a connection to the database. In the `Repository` derive, this generic connection is concretised to `PgPoolConnection`, which basically means we can use any client that can establish that connection. The `Postgres` client can do just that (it is simply a wrapper around a connection pool).

The `#[postgres(Conn)]` attribute tells the derive macro which generic connection parameter to substitute in the implementation and must match the generic in the struct. `RepositoryAccess` can be manually implemented as well.

`DBConnect` is a trait used by clients to establish an actual connection. All concrete clients implement it in their specific ways.
It is also implemented by the `Client` struct. A `Client` is a wrapper around a concrete client and simply delegates the `connect()` call to it.

As you can see, the client's `C` parameter MUST implement `DBConnect` which takes care of connecting to the database and its connection MUST be the same as the connection on `DBConnect`. This takes care of how we're connecting to the DB.

The `User` bound is simply a bound to a repository the service's repository will use, which in this case is the `UserRepository`. Since repository methods must take in a connection (in order to preserve transactions) they do not take in `&self`. This is fine, but now the compiler will complain we have unused fields because we are in fact not using them. If we remove the fields, the compiler will complain we have unused trait bounds, so we use phantom data to make the compiler think the struct owns the data.

So far we haven't coupled any implementation details to the service. The derive macro generates code for a postgres client, but it just substitutes the generic connection bounds for concrete ones in its `RepositoryAccess` implementation. It's called postgres because the derive macro has a specific set of fields on which it operates, this could be named anything in case of manual implementations.

So pretty much, all the service has are calls to some generic clients, connections and repositories.

This fact is at the core of this architecture and is precisely what makes it so powerful. Not only does this make testing a piece of cake, but it also allows us to switch up our adapters any way we want without ever having to change the business logic. They are completely decoupled.

Finally, we'll concretise everything in the setup:

- **setup.rs**

  ```rust
  pub(crate) fn routes(pg: Arc<Postgres>, rd: Arc<Redis>, cfg: &mut web::ServiceConfig) {
    let service = UserService {
        repository: Repository::<Postgres, PgPoolConnection, PgUserAdapter>::new(pg.clone()),
    };
    let auth_guard = interceptor::AuthGuard::new(pg, rd, Role::User);

    cfg.app_data(Data::new(service));

    // Show all
    cfg.service(
        web::resource("/users")
            .route(web::get().to(handler::get_paginated::<
                UserService<Repository<Postgres, PgPoolConnection, PgUserAdapter>>,
            >))
            .wrap(auth_guard),
    );
  }
  ```

I'll admit it, the trait bounds do look kind of ugly, but seeing as this is the only place where we concretise our types, we never have to worry about the rest of the service breaking when we makes changes in our adapters.

To reduce some of the unpleasentness with dealing with so many generics, macros exist to aid the process. If we utilise the `repository!` and `contract!` macro, our `adapter.rs` file becomes a bit more easy on the eyes:

- ***adapter.rs***

  ```rust
  /* ..imports.. */

  repository! {
      C => Connection : field_name;

      User => UserRepository<Connection>
  }
  
  contract! {
      C => Connection;

      RepositoryContract => Repository, RepositoryAccess;
      
      User => UserRepository<C>;
  
      fn get_paginated(
          &self,
          page: u16,
          per_page: u16,
          sort: Option<user::SortOptions>,
      ) -> Result<Vec<user::User>, Error> {
          let mut conn = self.connect()?;
          User::get_paginated(&mut conn, page, per_page, sort).map_err(Error::new)
      }
  }
  ```

Looks much better! This will essentially generate all the code with the generics from the original file.
You can read more about how the macros work in the `hextacy::db` module.

### **Transactions**

The reason for the repositories always taking in a connection in their methods is transactions. Since business level services should have the ability to rollback transactions if anything goes south, we have to somehow enable their repositories to suport transactions.

We do this by adding a transaction field to the repository, which is simply a `RefCell` around an `Option<C>` where `C` is the connection. We use the ref cell to get mutable access to transactions without poisoning our API with `&mut self` references.

This ref cell can now hold an open connection that can be used to perform queries. The `Atomic` trait provides an interface for any repository to start, commit or rollback a transaction. The way this is done is by checking whether our ref cell contains a connection, if it does we use that one and if it doesn't we simply instruct our client to establish a new one. Taking it one step further, let's make the user service repository atomic:

  ```rust
  use hextacy_derive::Repository;
  use hextacy::db::{AtomicConnection, Transaction};
  use hextacy::clients::db::{Client, DBConnect};
  use std::{marker::PhantomData, sync::Arc};
  
  #[derive(Debug, Clone, AcidRepository)]
  #[postgres(Conn)]
  pub struct Repository<C, Conn, User>
  where
      C: DBConnect<Connection = Conn>,
      User: UserRepository<Conn>,
  {
      pub postgres: Client<C, Conn>,
      // Type provided for convenience which is equivalent to RefCell<Option<Conn>>
      pub pg_tx: Transaction<Conn>,
      user: PhantomData<User>,
  }
  ```

Now, instead of simply establishing a connection and calling `User::get_paginated`, we first have to check whether an open connection exists:

  ```rust
  impl<C, Conn, User> RepositoryContract for Repository<C, Conn, User> 
  where
      Self: AcidRepositoryAccess<Conn>,
      C: DBConnect<Connection = Conn>,
      User: UserRepository<Conn>
  {
    fn get_paginated(
        &self,
        page: u16,
        per_page: u16,
        sort: Option<user::SortOptions>,
    ) -> Result<Vec<user::User>, Error> {
        let mut conn = self.connect()?;
        // Use atomic! to reduce this boilerplate
        match conn {
          hextacy::db::AtomicConnection::New(mut conn) => User::get_paginated(&mut conn, page, per_page, sort).map_err(Error::new),
          hextacy::db::AtomicConnection::Existing(mut conn) => User::get_paginated(conn.borrow_mut().as_mut().unwrap(), page, per_page, sort).map_err(Error::new),
        }
        
    }
  }
  ```

To reduce the boilerplate around matching whether a connection exists, the `atomic!` macro can be utilised to perform the query. It does exactly what's written above.

Notice that `Repository` is changed to `AcidRepository` and `RepositoryAccess` is changed to `AcidRepositoryAccess`. The access traits are the same, except that the atomic version returns an `AtomicConnection<C>` and requires the repository to implement `Atomic`, which `AcidRepository` does behind the scenes:

  ```rust
  use hextacy::db::{Atomic, DatabaseError, TransactionError};
  use diesel::connection::AnsiTransactionManager;

  impl</* ..bounds.. */> Atomic for Repository< /* ..bounds.. */, PgPoolConnection>
  where /* ..bounds.. */ 
  {
        fn start_transaction(&self) -> Result<(), DatabaseError> {
            let mut tx = self.transaction.borrow_mut();
            match *tx {
                Some(_) => Err(DatabaseError::Transaction(TransactionError::InProgress)),
                None => {
                    let mut conn = self.client.connect()?;
                    AnsiTransactionManager::begin_transaction(&mut *conn)?;
                    *tx = Some(conn);
                    Ok(())
                }
            }
        }

        fn rollback_transaction(&self) -> Result<(), DatabaseError> {
            let mut tx = self.transaction.borrow_mut();
            match tx.take() {
                Some(ref mut conn) => AnsiTransactionManager::rollback_transaction(&mut **conn)
                    .map_err(DatabaseError::from),
                None => Err(DatabaseError::Transaction(TransactionError::NonExisting).into()),
            }
        }

        fn commit_transaction(&self) -> Result<(), DatabaseError> {
            let mut tx = self.transaction.borrow_mut();
            match tx.take() {
                Some(ref mut conn) => {
                    AnsiTransactionManager::commit_transaction(&mut **conn)
                        .map_err(DatabaseError::from)
                }
                None => Err(DatabaseError::Transaction(TransactionError::NonExisting).into()),
            }
        }
    }
  ```

Atomic implementations need to have concrete types since it must know which transaction manager to use to operate on the connection.

Thankfully, `AcidRepository` does this for us. One more shorcut that can be used is the `acid_repo!` macro which functions the same as `repository` except with the addition of the transaction field and the atomic access implementation.

Business level services can now utilise the three methods to perform transactions as they see fit. To reduce the boilerplate associated with them, we can utilise the `transaction!` macro.

This macro takes in a callback that must return a result. Before the callback start, `start_transaction` will be called, then, depending on the result, the transaction will either be committed or rollbacked.

To elaborate further, here's what a repository would look like:

- **repository/user.rs**

```rust
pub trait UserRepository<C> {
    fn get_paginated(
        conn: &mut C,
        page: u16,
        per_page: u16,
        sort_by: Option<SortOptions>,
    ) -> Result<Vec<User>, AdapterError>;
}
```

The adapter just implements the `UserRepository` trait and returns the model using its specific ORM. This concludes the architectural part (for now... :).

## **hextacy**

Contains various utilities for working with http, email and websockets:

- ### **db**

  Contains a collection of traits to implement on structures that access databases and interact with repositories. Provides macros to easily generate repository structs as shown in the example.

- ### **clients**
  
  Contains structures implementing client specific behaviour such as connecting to and establishing connection pools with database, cache, smtp and http servers. All the connections made here are generally shared throughout the app with Arcs.

- ### **logger**

  The `logger` module utilizes the [tracing](https://docs.rs/tracing/latest/tracing/), [env_logger](https://docs.rs/env_logger/latest/env_logger/) and [log4rs](https://docs.rs/log4rs/latest/log4rs/) crates to setup logging either to stdout or a `server.log` file, whichever suits your needs better.
  
- ### **crypto**

  Contains cryptographic utilities for encrypting and signing data and generating tokens.

- ### **web**

  Contains various helpers and utilities for HTTP and websockets.

  - **http**

      The most notable here are the *Default security headers* middleware for HTTP (sets all the recommended security headers for each request as described [here](https://www.npmjs.com/package/helmet)) and the *Response* trait, a utility trait that can be implemented by any struct that needs to be turned in to an HTTP response. Also some cookie helpers.

  - **ws**

      Module containing a Websocket session handler.

      Every message sent to this handler must have a top level `"domain"` field. Domains are completely arbitrary and are used to tell the ws session which datatype to broadcast.

      Domains are internally mapped to data types. Actors can subscribe via the broker to specific data types they are interested in and WS session actors will in turn publish them whenever they receive any from their respective clients.

      Registered data types are usually enums which are then matched in handlers of the receiving actors. Enums should always be untagged, so as to mitigate unnecessary nestings from the client sockets.

      Uses an implementation of a broker utilising the [actix framework](https://actix.rs/book/actix/sec-2-actor.html), a very cool message based communication system based on the [Actor model](https://en.wikipedia.org/wiki/Actor_model).

      Check out the `web::ws` module for more info and an example of how it works.

- ### **cache**

  Contains a cacher trait which can be implemented for services that require access to the cache. Each service must have its cache domain and identifiers for cache seperation. The `CacheAccess` and `CacheIdentifier` traits can be used for such purposes.

### **A note on middleware**
  
  The structure is similar to the endpoints as demonstrated above. If you're interested in a bit more detail about how Actix's middleware works, [here's a nice blog post you can read](https://imfeld.dev/writing/actix-web-middleware). By wrapping resources with middleware we get access to the request before it actually hits the handler. This enables us to append any data to the request for use by the designated handler. Essentially, we have to implement the `Transform` trait for the middleware and the `Service` trait for the actual business logic.

  If you take a look at the `auth` middleware you'll notice how our `Transform` implementation, specifically the `new_transform` function returns a future whose output value is a result containing either the `AuthMiddleware` or an `InitError` which is a unit type. If you take a look at the signature for Actix's `wrap` function you can see that we can pass to it anything that implements `Transform`. This means that, for example, when we want to wrap a resource with our `AuthGuardMiddleware`, we have to pass the instantiated `AuthGuard` struct, because that's the one implementing `Transform`.
  If you take an even closer look at what happens in `wrap` you'll see that it triggers `new_transform` internally, meaning the instantiated `AuthGuard` transforms into an `AuthGuardMiddleware` which executes all the business.

  The structure is exactly the same as that of endpoints with the exception of **interceptor.rs** which contains our `Transform` and `Service` implementations. The main functionality of the middleware is located in the `call` function of the `Service` implementation.

### **The config file**

  We tie all our handlers together in the `config.rs` file in the server's `src` directory. With only this one endpoint it would look something like:

  ```rust
  pub(super) fn init(cfg: &mut ServiceConfig) {
      let pg = Arc::new(Postgres::new());

      users::setup::routes(pg, cfg);
  }
  ```

  We would then pass this function to our server setup.

  ```rust
      HttpServer::new(move || {
          App::new()
              .configure(config::init)
              .wrap(Logger::default())
      })
      .bind_openssl(addr, builder)?
      .run()
      .await
  ```

  Read more about the openssl setup in `openssl/README.md`

The helpers module contains various helper functions usable throughout the server.

### **Storage Directory Overview**

The storage crate is project specific which is why it's completely seperated from the rest. It contains 3 main modules:

- **Repository**

    Contains interfaces for interacting with application models. Their sole purpose is to describe the nature of interaction with the database, they are completely oblivious to the implementation. This module is designed to be as generic as possible and usable anywhere in the service logic.

- **Adapters**

    Contains the client specific implementations of the repository interfaces. Adapters adapt the behaviour dictated by their underlying repository. Seperating implementation from behaviour decouples any other module using a repository from the client specific code located in the adapter.

- **Models**

    Where application models are located.

The storage adapters can utilize connections established from the clients module.

## **Authentication flow**

The user is expected to enter their email and password after which an email with a registration token gets sent (`start_registration`).

Users can request another token if their token expires (`resend_registration_token`).

Once the user verifies their registration token they must log in, after which they will receive a session ID cookie and a CSRF token in the header (`verify_registration_token`, `login`).

The cookie and token are then used by the middleware to authenticate the user when accessing protected resources. It does so by grabbing both from the request and trying to fetch a session first from the cache, then if that fails from postgres. The session is searched for by ID and must be unexpired and have a matching csrf token, otherwise the middleware will error.

A route exists for setting a user's OTP secret and a session must be established to access it (`set_otp_secret`).

When a user sets their OTP secret they have to provide a valid OTP after successfully verifying credentials or they won't be able to establish a session (`verify_otp`).

Users can change their password and logout only if they have an established session. On logout a user can also choose to purge all of their sessions (`change_password`, `logout`).

If a user changes their password their sessions will be purged and they will receive an email notifying them of the change with a password reset token in case it wasn't them. The PW reset token lasts for 2 days (`reset_password`).

Users who forgot their passwords can request a password reset. They will receive an email with a temporary token they must send upon changing their password for the server to accept the change. Once they successfully change it their sessions will be purged and a new one will be established (`forgot_password`, `verify_forgot_password`).

## **XTC**

A.K.A. the CLI tool provides a way of seamlessly generating and documenting endpoints and middleware.

To set up the cli tool after cloning the repository enter

```bash
cargo install --path xtc
```

from the project root.

The list of top level commands can be viewed with the `xtc -h` command.

The most notable commands are `[g]enerate` which sets up endpoint/middleware boilerplate and `[anal]yze` which scans the router and middleware directories and constructs a Json/Yaml file containing endpoint info.

Xtc only works for the project structure described in [the architecture section](#the-server).

The `[g]enerate` command generates an endpoint structure like the one described in the router. It can generate `route [r]` and `middleware [mw]` boilerplate. Contracts can also supplied to the command with the `-c` flag followed by the contracts you wish to hook up to the endpoint, comma seperated e.g.

```bash
xtc gen route <NAME> -c repository,cache
```

This will automagically hook up the contracts to the service and set up an infrastructure boilerplate. It will also append `pub(crate) mod <NAME>` to the router's `mod.rs`. It also takes in a `-p` argument which can be used to specify the directory you want to set up the endpoint.

The `analyze` function heavily relies on the [syn crate](https://docs.rs/syn/latest/syn/). It analyzes the syntax of the `data`, `handler` and `setup` files and extracts the necessary info to document the endpoint.

All commands take in the `-v` flag which stands for 'verbose' and if true print what xtc is doing to stdout. By default, all commands are run quietly.

TODO:

- [ ] Add maxmind and activity logging middleware

- [ ] Openssl with let's encrypt

- [ ] Init project with `xtc init`

- [ ] Something probably
