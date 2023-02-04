# ⚗️ Alchemy

A repo designed to quick start web server development with [actix_web](https://actix.rs/) by providing an extensible infrastructure and a CLI tool to reduce manually writing boilerplate while maintaining best practices.

The kind of project structure this repo uses is heavily based on [hexagonal architecture](https://en.wikipedia.org/wiki/Hexagonal_architecture_(software)) also known as the *ports and adapters* architecture which is very flexible and easily testable. You can read great articles about it [here](https://netflixtechblog.com/ready-for-changes-with-hexagonal-architecture-b315ec967749) and [here](https://blog.phuaxueyong.com/post/2020-05-25-what-architecture-is-netflix-using/).

## **Get started**

Create a real `.env` file filling out the example file with your desired parameters and create the database you entered in the file. You do not have to fill out fields you don't intend to use.

For the Email part, use [this spec](https://support.google.com/mail/answer/7126229?hl=en#zippy=%2Cstep-change-smtp-other-settings-in-your-email-client) to set up the SMTP host(smtp.gmail.com) and port(465) and [follow these instructions](https://support.google.com/accounts/answer/185833?hl=en#zippy=%2Cwhy-you-may-need-an-app-password) to generate an app password. The password can then be used for the `SMTP_PASSWORD` variable. For the sender and username enter your email address.

For the secrets, run

```bash
cargo run -p infrastructure --bin generate_secret <SECRET_NAME>
```

and replace them with the example ones.

Install the CLI tool via

```bash
cargo install --path alx
```

Run migrations via

```bash
alx m run
```

Give yourself a pat on the back since you've made it this far and optionally check out all the commands with `alx -h`.

Run the server with

```bash
cargo run -p server
```

and load the postman collection located in `misc` to interact with it.

You now have a ready to go infrastructure, now go make your billion $ app!

## **Server**

The main binary.

The `main.rs` is where the server gets instantiated. The `configure` file is used to set up all the infrastructure connections and endpoints. The `Error` enum from `error.rs` is a wrapper around external errors that implements actix's `ErrorResponse` trait, meaning we can send any error we encounter as a custom HTTP response.

### **API**

  The heart of the server. This is where all the domain logic is implemented for each endpoint located in the ***router***, as well as the ***middleware*** you define for incoming requests.

- ### **Router**
  
  The router contains the endpoints of the server. The endpoints provide a compact way of writing your business logic all in one place. Usually, an endpoint will consist of 6 main files:
  
  - **contract.rs**
  
    A contract specifies certain conditions the endpoint's domain/infrastructure must fulfil.
  
    A simple example for a very basic user service would look like:
  
    ```rust
    pub(super) trait ServiceContract {
      fn get_paginated(&self, data: GetUsersPaginated) -> Result<HttpResponse, Error>;
    }
    ```
  
    Here we wrtie only the signatures we want our endpoint service to have. By having an `HttpResponse` in the return signature we retain the flexibility of responding with different responses instead of having a concrete type to return from the service. This essentially allows us to return any struct that implements the `Response` trait.
  
  - **data.rs**

    Here we define the data we expect to receive and intend to send as responses. An example for a user request and response would look something like:
  
    ```rust
    // We expect this in the query params
    #[derive(Debug, Deserialize)]
    // Validify creates a GetUsersPaginatedPayload in the background
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
  
    We also specify how the inputs get validated by the [validify](https://crates.io/crates/validify) crate. The response trait is a utility trait for converting this struct to a response, as you'll see in the domain.
  
  - **infrastructure.rs**
  
    Here we define the specific implementations of contracts intended to utilise alx_core services.
    For simple services such as this, the file can be omitted (our adapters contain the implementations for communicating with the database, so no need to write it here as well). If we, for example, had some kind of caching functionality in the service, we would write the implementation here.

    Check out the `auth` endpoint from the server to see what it would actually look like.
  
  - **domain.rs**
  
    Here we define the domain logic, i.e. we implement the behaviour we want this service to have when we get a request for this route. This part is where all of the infrastructure pieces tie together and perform the business. This is where we implement the `ServiceContract` for this module's `UserService` struct.
  
    ```rust
    use super::contract::ServiceContract;
    use infrastructure::storage::repository::user::UserRepository;
  
    pub(super) struct UserService<R: UserRepository> {
      pub user_repo: R,
    }
  
    impl<R> ServiceContract for UserService<R>
    where
      R: UserRepository + Send + Sync,
    {
      fn get_paginated(&self, data: GetUsersPaginated) -> Result<HttpResponse, Error> {
          let users = self
              .user_repo
              .get_paginated(
                  data.page.unwrap_or(1_u16),
                  data.per_page.unwrap_or(25),
              )
              .await?;
  
          Ok(UserResponse::new(users).to_response(StatusCode::OK, None, None))
      }
    }
    ```
  
     Notice how we didn't tie any implementation to the repository, it can take in anything as long as it implements `UserRepository`. This allows for easy mocking of the function and seperation of the business logic of the service from the implementation details of our repository adapters. In this service's implementation of the `get_paginated` function we just call our repository to get a list of users and return them in the `UserResponse` from our `data` module. Here the `to_response()` call converts the struct to an HTTP response with a 200 OK status code, no cookies and no additional headers.
  
  - **handler.rs**
  
    Here we define our request handlers. These are the entry points for the domain of each endpoint.
  
    ```rust
    use super::{contract::ServiceContract, data::GetUsersPaginatedPayload};
  
    pub(super) async fn get_paginated<S: ServiceContract>(
      data: web::Query<GetUsersPaginatedPayload>,
      service: web::Data<S>,
    ) -> Result<impl Responder, Error> {
      data.0.validate().map_err(Error::new)?;
      info!("Getting users");
      service.get_paginated(data.0).await
    }
    ```
  
    We place a `ServiceContract` trait bound on the handler so we can later inject the specific service configuration we want. We expect to receive the necessary data in the query parameters and validate it. We plug the service we intend to use in the `service` function parameter, then we call the service's `get_paginated` function which returns either an error or a successful HTTP response, as defined in the domain.
  
  - **setup.rs**
  
    This is where we instantiate the service and hook everything up.
  
    ```rust
    use super::{domain::UserService, handler, infrastructure::Repository};
  
    pub(crate) fn routes(pg: Arc<Postgres>, cfg: &mut web::ServiceConfig) {
      let service = UserService {
            user_repo: PgUserAdapter { client: pg.clone() },
      };
  
      cfg.app_data(Data::new(service));
  
      cfg.service(
          web::resource("/users")
              .route(web::get().to(handler::get_paginated::<UserService<PgUserAdapter>>))
      );
    }
  
    ```
  
    This function needs to be `pub(crate)` as it gets used by the router module. Here we pass in the Arcs to the connection pools we want to use and actix's `ServiceConfig`. We then construct the service with the specific adapter we want to use for the user repository, pass it the connection pool, wrap the service in actix's `Data` wrapper and configure the server to use it, set the '/users' resource to call the handler we specified for this route and inject the `PgUserAdapter` as our repository. Also note that this is the only file in which we specify the exact adapter to plug in to the service.
  
- ### **Middleware**
  
  Contains the middleware used by the server for intercepting HTTP requests. The structure is very similar to the router endpoints. If you're interested in a bit more detail about how Actix's middleware works, [here's a nice blog post you can read](https://imfeld.dev/writing/actix-web-middleware). By wrapping resources with middleware we get access to the request before it actually hits the handler. This enables us to append any data to the request for use by the designated handler. Essentially, we have to implement the `Transform` trait for the middleware and the `Service` trait for the actual business logic.

  If you take a look at the `auth` middleware you'll notice how our `Transform` implementation, specifically the `new_transform` function returns a future whose output value is a result containing either the `AuthMiddleware` or an `InitError` which is a unit type. If you take a look at the signature for Actix's `wrap` function you can see that we can pass to it anything that implements `Transform`. This means that, for example, when we want to wrap a resource with our `AuthGuardMiddleware`, we have to pass the instantiated `AuthGuard` struct, because that's the one implementing `Transform`.
If you take an even closer look at what happens in `wrap` you'll see that it triggers `new_transform` internally, meaning the instantiated `AuthGuard` transforms into an `AuthGuardMiddleware` which executes all the business.

  The structure is exactly the same as that of endpoints with the exception of **interceptor.rs** which contains our `Transform` and `Service` implementations. The main functionality of the middleware is located in the `call` function of the `Service` implementation.

- ### **config**

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

The helpers module contains various helper functions usable throughout the server.

The benefits of having this kind of architecture start to become clear once your application gets more complex. With only one user repository it might seem like overkill at first, but imagine you have some kind of service that communicates with multiple repositories, the cache and email (e.g. the authentication module from this starter kit). Things would quickly get out of hand. This kind of structure allows for maximum flexibility in case of changes and provides a readable file of all the business logic (`contract.rs`) and the data we expect to manipulate (`data.rs`).

If your logic gets complex, you can split the necessary files to directories and seperate the logic there. The rust compiler will warn you that you need to change the visibilites of the data if you do this. It's best to keep everything scoped at the endpoint level except for `setup.rs`, which should be scoped at `api` level since we need it in `config.rs`.

## **Storage**

The storage crate is project specific which is why it's completely seperated from the rest. It contains 3 main modules:

- **Repository**

    Contains interfaces for interacting with application models. Their sole purpose is to describe the nature of interaction with the database, they are completely oblivious to the implementation. This module is designed to be as generic as possible and usable anywhere in the domain logic.

    A very simple repository would look something like:

    ```rust
    pub trait UserRepository {
      fn get_paginated(
          &self,
          page: u16,
          per_page: u16,
          sort_by: Option<SortOptions>,
      ) -> Result<Vec<User>, RepositoryError>;
    }
    ```

- **Adapters**

    Contains the client specific implementations of the repository interfaces. Adapters adapt the behaviour dictated by their underlying repository. Seperating implementation from behaviour decouples any other module using a repository from the client specific code located in the adapter.

- **Models**

    Where application models are located.

The storage adapters utilize connections established from the clients module:

## **Clients**
  
  Contains structures implementing client specific behaviour such as connecting to and establishing connection pools with database, cache, smtp and http servers. All the connections made here are generally shared throughout the app with Arcs.

## **Logger**

The `logger` module utilizes the [tracing](https://docs.rs/tracing/latest/tracing/), [env_logger](https://docs.rs/env_logger/latest/env_logger/) and [log4rs](https://docs.rs/log4rs/latest/log4rs/) crates to setup logging either to stdout or a `server.log` file, whichever suits your needs better.

## **Core**

Contains various utilities for working with http, email and websockets:
  
- ### **Crypto**

  Contains cryptographic utilities for encrypting and signing data and generating tokens.

- ### **Web**

  Contains various helpers and utilities for HTTP and websockets.

  - **http**

      The most notable here are the *Default security headers* middleware for HTTP (sets all the recommended security headers for each request as described [here](https://www.npmjs.com/package/helmet)) and the *Response* trait, a utility trait that can be implemented by any struct that needs to be turned in to an HTTP response.

  - **ws**

      Module containing a Websocket session handler.

      Every message sent to this handler must have a top level `"domain"` field. Domains are completely arbitrary and are used to tell the ws session which datatype to broadcast.

      Domains are internally mapped to data types. Actors can subscribe via the broker to specific data types they are interested in and WS session actors will in turn publish them whenever they receive any from their respective clients.

      Registered data types are usually enums which are then matched in handlers of the receiving actors. Enums should always be untagged, so as to mitigate unnecessary nestings from the client sockets.

      Uses an implementation of a broker utilising the [actix framework](https://actix.rs/book/actix/sec-2-actor.html), a very cool message based communication system based on the [Actor model](https://en.wikipedia.org/wiki/Actor_model).

      Check out the `web::ws` module for more info and an example of how it works.

- ### **Cache**

  Contains a cacher trait which can be implemented for services that require access to the cache. Each service must have its cache domain and identifiers for cache seperation.

## **Mocking**

Mocking allows us to test the business logic of our domains. With this type of architecture mocking is easy and efficient as it contains almost no implementation details. In mock tests we utilize the [mockall](https://docs.rs/mockall/latest/mockall/) crate. This crate allows us to instantiate our services with mock versions of our `Contract` implementations. To understand what's going on when we use mockall it's best to see an example:

  To mock our service and repository contracts we have to annotate them with the `automock` attribute

  ```rust
  #[cfg_attr(test, mockall::automock)]
  #[async_trait]
  pub(super) trait ServiceContract {
    async fn get_paginated(&self, data: GetUsersPaginated) -> Result<HttpResponse, Error>;
  }
  ```

  The same applies for the `RepositoryContract`. The `cfg_attr` with the test flag means these mock implementations will only be available in a `#[cfg(test)]` module.

  Our mocks are located in the `mod.rs` file of an endpoint. Here we define what we *expect* to happen once an endpoint function triggers. For our simple paginated users function this would look like:

  ```rust
  let mut repository = MockUserRepository::new();

  repository.expect_get_paginated().return_once(|_,_| Ok(vec![MOCK_USER.clone()]));

  let data = GetUsersPaginated {
    page: 1,
    per_page: 25
  };

  let service = UserService { repository };
  service.get_paginated(data).await.unwrap();
  ```

  We instantiate a mock repository and set an expectation. In our simple handler we only expect the repository to get a paginated list of users so that's the only thing we have to expect. When a service's function contains multiple calls to its infrastructure contracts we would expect them all in the test.
  Notice also that we instantiated the real deal service at the end and not a mock one.
  
  If the service also has calls to itself (i.e. `self.do_something()` as opposed to `self.repository.do_something()`), we would have to mock the service contract as well and expect everything that would get triggered in the function call.

  The `MOCK_USER` is a static lazy loaded `User` struct which we can reuse in our tests to prevent us from having to instantiate a user in every test.

  The beauty here is that we can return anything in our expectations so long as it matches the contract's signature. For example, since `get_paginated` returns a result, instead of returning an OK vec with MOCK_USER, we could have just as easily returned an error

  ```rust
  repository.expect_get_paginated().return_once(|_,_| Err(Error::/* Some error */));
  ```

  which would then trigger flows which would happen if the function actually errored in runtime.

  For the first example we can just unwrap the result since our test would fail if it was an error and we'd know something's wrong. For the second example we can even inspect and assert that we got the right error:

  ```rust
  let result = service.get_paginated(data).await;
  match result {
    Ok(_) => panic!("Should not have happened"),
    Err(e) => assert!(matches!(e, Error::/* Some error */)),
  }
  ```

  Just keep in mind that the errors returned by the service will be the ones you specify in the domain.

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

## **Alx**

A.K.A. the CLI tool provides a way of seamlessly generating and documenting endpoints and middleware.

To set up the cli tool after cloning the repo enter

```bash
cargo install --path alx
```

from the project root.

The list of top level commands can be viewed with the `alx -h` command.

The most notable commands are `[g]enerate` which sets up endpoint/middleware boilerplate and `[anal]yze` which scans the router and middleware directories and constructs a Json/Yaml file containing endpoint info.

Alx only works for the project structure described in [the router](#router).

The `[g]enerate` command generates an endpoint structure like the one described in the router. It can generate `route [r]` and `middleware [mw]` boilerplate. Contracts can also supplied to the command with the `-c` flag followed by the contracts you wish to hook up to the endpoint, comma seperated e.g.

```bash
alx gen route <NAME> -c repo,cache
```

This will automagically hook up the contracts to the domain service and set up an infrastructure boilerplate. It will also append `pub(crate) mod <NAME>` to the router's `mod.rs`. It also takes in a `-p` argument which can be used to specify the directory you want to set up the endpoint.

The `analyze` function heavily relies on the [syn crate](https://docs.rs/syn/latest/syn/). It analyzes the syntax of the `data`, `handler` and `setup` files and extracts the necessary info to document the endpoint.

All commands take in the `-v` flag which stands for 'verbose' and if true print what alx is doing to stdout. By default, all commands are run quietly.

TODO:

- [ ] Add maxmind and activity logging middleware

- [ ] Oauth stuff

- [ ] Openssl with let's encrypt

- [ ] Init project with `alx init`

- [ ] Something probably
