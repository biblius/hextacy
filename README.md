# Rust Web Starter

This repo is deisgned to quick start web server development with [actix_web](https://actix.rs/) by providing out of the box basic user authentication flows, session middleware, a simple CLI tool for file managment and a bunch of utilities for the web.

This kind of project structure is heavily based on [hexagonal architecture](https://en.wikipedia.org/wiki/Hexagonal_architecture_(software)) also known as the *ports and adapters* architecture which is very flexible and easily testable. You can read great articles about it [here](https://netflixtechblog.com/ready-for-changes-with-hexagonal-architecture-b315ec967749) and [here](https://blog.phuaxueyong.com/post/2020-05-25-what-architecture-is-netflix-using/).

The heart of this repository are the infrastructure and server directories.

--

## **Infrastructure**

Contains the foundations from which we build our servers. Here you'll find all the database clients, adapters and repositories as well as a bunch of crypto, web and actor helpers.

The most interesting here is the store module, where the said data sources are located. It is divided in to three parts:

### **Store**

- #### **Repository**

  Contains data structures and the interfaces with which we interact with them. Their sole purpose is to describe the nature of interaction with the database, they are completely oblivious to the implementation. This module is designed to be as generic as possible and usable anywhere in the domain logic.

- #### **Adapters**

  Contains the client specific implementations of the repository interfaces. Adapters adapt the behaviour dictated by their underlying repository. Seperating implementation from behaviour decouples any other module using a repository from the client specific code located in the adapter.

- #### **Models**

  This is where application models are located. These aren't necessarily meant to be stored in databases and serve as utility structures for responses, the cache and intermediary data that can be used across the project.

The store adapters utilize connections established from the clients module:

### **Clients**

Contains structures implementing client specific behaviour such as connecting to and establishing connection pools with database, cache, smtp and http servers. All the connections made here are generally shared throughout the app with Arcs.

### **Actors**

Module containing an implementation of a basic broadcastable message and a broker utilising the [actix framework](https://actix.rs/book/actix/sec-2-actor.html), a very cool message based communication system based on the [Actor model](https://en.wikipedia.org/wiki/Actor_model).

The rest is a bunch of helpers that don't require that much explanation. We have the **config** directory which just loads and sets environment variables, the **crypto** directory containing cryptographic utilities for encrypting, signing and generating tokens and the **web** directory containing various helpers and utilities for HTTP and websockets. The most notable modules from **web** are the *Default security headers* middleware for HTTP (sets all the recommended security headers for each request as described [here](https://www.npmjs.com/package/helmet)), the *Response* trait, a utility trait that can be implemented by any struct that needs to be turned in to an HTTP response and a websocket actor useful for maintaing a websocket session.

## **Server**

The main binary. Contains the domain logic and the request handlers.

The `main.rs` is where the server gets instantiated. The `Error` enum from `error.rs` is a wrapper around external errors that implements actix's `ErrorResponse` trait, meaning we can send any error we encounter as a custom HTTP response.

### **API**

The heart of the server. This is where all the domain logic is implemented for each endpoint located in the ***router***, as well as the ***middleware*** you define for incoming requests.

#### **Router**

The router contains the endpoints of the server. The endpoints provide a compact way of writing your business logic all in one place. Usually, an endpoint will consist of 7 files:

- #### **contract.rs**

  Like the repositories from the infrastructure, this is where the behaviour for this endpoint is described. A contract specifies certain conditions the endpoint's domain/infrastructure must fulfil.

  A simple example for a very basic user service would look like:

  ```rust
  #[async_trait]
  pub(super) trait ServiceContract {
    async fn get_paginated(&self, data: GetUsersPaginated) -> Result<HttpResponse, Error>;
  }

  #[async_trait]
  pub(super) trait RepositoryContract {
    async fn get_paginated(
        &self,
        page: u16,
        per_page: u16,
    ) -> Result<Vec<User>, Error>;
  }
  ```

  No implementation details are written here, only the signatures we want our endpoint service to have. By having an `HttpResponse` in the return signature we retain the flexibility of responding to the user with different responses instead of having a concrete type to return from the service. This essentially allows us to return any struct that implements the `Response` trait.

- #### **data.rs**
  
  Here we define the data we expect to receive and intend to send as responses. An example for a user request and response would look something like:

  ```rust
  // We expect this in the query params
  #[derive(Debug, Deserialize, Validate)]
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

  We also specify how the inputs get validated by the [validator](https://docs.rs/validator/latest/validator/) crate. The response trait is a utility trait for converting this struct to a response, as you'll see in the domain.

- #### **infrastructure.rs**

  Here we define the specific implementation of the aformentioned `RepositoryContract` for this service's repository.

  ```rust
  use super::contract::RepositoryContract;

  pub(super) struct Repository<UR>
  where
    UR: UserRepository,
  {
    pub user_repo: UR,
  }

  #[async_trait]
  impl<UR> RepositoryContract for Repository<UR>
  where
    UR: UserRepository<Error = PgAdapterError> + Send + Sync,
  {
    async fn get_paginated(
        &self,
        page: u16,
        per_page: u16,
        sort_by: Option<SortOptions>,
    ) -> Result<Vec<User>, Error> {
        self.user_repo
            .get_paginated(page, per_page, sort_by)
            .await
            .map_err(|e| AdapterError::Postgres(e).into())
    }
  }
  ```

  Here, `UserRepository` is again seperated from the business logic as we can plug in any adapter that implements it. In our case we are using the `PgUserAdapter`, a struct containing a postgres specific implementation and we define the error type to be a `PgAdapterError` as `Repository` requires a specific error for each implementation.

- #### **domain.rs**

  Here we define the domain logic, i.e. we implement the behaviour we want this service to have when we get a request for this route. This part is where all of the infrastructure pieces tie together and perform the business. This is where we implement the `ServiceContract` for this module's `UserService` struct.

  ```rust
  use super::{
    contract::{RepositoryContract, ServiceContract},

  pub(super) struct UserService<R: RepositoryContract> {
    pub repository: R,
  }

  #[async_trait]
  impl<R> ServiceContract for UserService<R>
  where
    R: RepositoryContract + Send + Sync,
  {
    async fn get_paginated(&self, data: GetUsersPaginated) -> Result<HttpResponse, Error> {
        let users = self
            .repository
            .get_paginated(
                data.page.unwrap_or(1_u16),
                data.per_page.unwrap_or(25),
            )
            .await?;

        Ok(UserResponse::new(users).to_response(StatusCode::OK, None, None))
    }
  }
  ```

   Notice how we didn't tie any implementation to the repository, it can take in anything as long as it implements the `RepositoryContract` from the `contract` module. This allows for easy mocking of the function and seperation of the business logic of the service from the implementation details of our repository adapters. In this service's implementation of the `get_paginated` function we just call our repository to get a list of users and return them in the `UserResponse` from our `data` module. Here the `to_response()` call converts the struct to an HTTP response with a 200 OK status code, no cookies and no additional headers.

- #### **handler.rs**

  Here we define our request handlers. These are the entry points for the domain of each endpoint.

  ```rust
  use super::{contract::ServiceContract, data::GetUsersPaginated};

  pub(super) async fn get_paginated<S: ServiceContract>(
    data: web::Query<GetUsersPaginated>,
    service: web::Data<S>,
  ) -> Result<impl Responder, Error> {
    data.0.validate().map_err(Error::new)?;
    info!("Getting users");
    service.get_paginated(data.0).await
  }
  ```

  We place a `ServiceContract` trait bound on the handler so we can later inject the specific service configuration we want. We expect to receive the necessary data in the query parameters and validate it. We plug the service we intend to use in the `service` function parameter, then we call the service's `get_paginated` function which returns either an error or a successful HTTP response, as defined in the domain.

- #### **setup.rs**

  This is where we instantiate the service and hook everything up.

  ```rust
  use super::{domain::UserService, handler, infrastructure::Repository};

  pub(crate) fn routes(pg: Arc<Postgres>, cfg: &mut web::ServiceConfig) {
    let service = UserService {
        repository: Repository {
            user_repo: PgUserAdapter { client: pg.clone() },
        },
    };

    cfg.app_data(Data::new(service));

    cfg.service(
        web::resource("/users")
            .route(web::get().to(handler::get_paginated::<UserService<Repository<PgUserAdapter>>>))
    );
  }

  ```

  This function needs to be `pub(crate)` as it gets used by the router module. Here we pass in the Arcs to the connection pools we want to use and actix's `ServiceConfig`. We then construct the service with the specific adapter we want to use for the user repository, pass it the connection pool, wrap the service in actix's `Data` wrapper and configure the server to use it, set the '/users' resource to call the handler we specified for this route and inject the `PgUserAdapter` as our repository.

#### **Middleware**

Contains the middleware used by the server for intercepting HTTP requests. The structure is very similar to the router endpoints. If you're interested in a bit more detail about how Actix's middleware works, [here's a nice blog post you can read](https://imfeld.dev/writing/actix-web-middleware). By wrapping resources with middleware we get access to the request before it actually hits the handler. This enables us to append any data to the request for use by the designated handler. Essentially, we have to implement the `Transform` trait for the middleware and the `Service` trait for the actual business logic.

If you take a look at the `auth` middleware you'll notice how our `Transform` implementation, specifically the `new_transform` function returns a future whose output value is a result containing either the `AuthMiddleware` or an `InitError` which is a unit type. If you take a look at the signature for Actix's `wrap` function you can see that we can pass to it anything that implements `Transform`. This means that whenever we want to wrap a resource with a middleware, we have to pass the instantiated `AuthGuard` struct, because that's the one implementing `Transform`.
If you take an even closer look at what happens in `wrap` you'll see that it triggers `new_transform` internally, meaning the instantiated `AuthGuard` transforms into an `AuthGuardMiddleware` which executes all the business.

The structure is exactly the same as that of endpoints with the exception of **interceptor.rs** which contains our `Transform` and `Service` implementations. The main functionality of the middleware is located in the `call` function of the `Service` implementation.

--

The benefits of having this kind of architecture start to become clear once your application gets more complex. With only one user repository it might seem like overkill at first, but imagine you have some kind of service that communicates with multiple repositories, the cache and email (e.g. the authentication module from this starter kit). Things would quickly get out of hand. This kind of structure allows for maximum flexibility in case of changes and provides a readable file of all the business logic (`contract.rs`) and the data we expect to manipulate (`data.rs`).

If your logic gets complex, you can split the necessary files to directories and seperate the logic there. The rust compiler will warn you that you need to change the visibilites of the data if you do this, it's best to keep the visibility public only at the endpoint directory and this can be achieved with with `pub(in path)` where path is the module where you want it to be visible, e.g. for one level of nesting it would be `pub(in super::super)`

We tie all our handlers together in the `mod.rs` file of the router. With only this one endpoint it would look something like:

```rust
pub fn init(
    pg: Arc<Postgres>,
    cfg: &mut ServiceConfig,
) {
    users::setup::routes(pg, cfg);
}

```

We would then pass this function to our server setup.

```rust
  let pg = Arc::new(Postgres::new());

    HttpServer::new(move || {
        App::new()
            .configure(|cfg| router::init(pg.clone(), cfg))
            .wrap(Logger::default())
    })
    .bind_openssl(addr, builder)?
    .run()
    .await
```

Obviously a real project would have much more routes and passing all the arcs to one function would be crazy, so we would seperate that into a `configure.rs` module where we'd set up all our clients and call that function instead of initializing everything in `main.rs`.

The helpers module contains various helper functions usable throughout the server.

## **Authentication flow**

The user is expected to enter their email and password after which an email with a registration token gets sent. Users can request another token if their token expires. Once the user verifies their registration they must log in, after which they will receive a session ID cookie and a CSRF token in the header.

The cookie and token are then used by the middleware to authenticate the user when accessing protected resources. It does so by grabbing both from the request and trying to fetch a session first from the cache, then if that fails from postgres. The session is searched for by ID and must be unexpired and have a matching csrf token, otherwise the middleware will error.

There is a predefined route for setting a user's OTP secret, a session must be established to do so. When a user sets their OTP secret they have to provide a valid OTP after successfully verifying credentials or they won't be able to establish a session.

Users can change their password and logout only if they have an established session. If a user changes their password they receive an email notifying them of the change with a password reset token in case it wasn't them, the PW reset token lasts for 2 days. On logout a user can purge all of their sessions.

Users who forgot their passwords can request a password reset. They will receive an email with a temporary token they must send upon changing their password for the server to accept the change. Once they successfully change it they .

## **CLI Tool**

TBD

TODO:

- [ ] Add maxmind and activity logging middleware

- [ ] Finish CLI tool

- [ ] Project config file for ez endpoint setup

- [ ] Openssl with let's encrypt

- [ ] Something I'm probably forgetting right now
