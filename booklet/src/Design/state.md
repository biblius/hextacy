# State

So now that we have the application core, a way to talk to it, and a way for it to obtain the data, we can now tie everything together.

We declare a `State` struct in which we keep references to concrete drivers, the concrete constructor for our service, and we move the concrete type alias here.

```rust
use hextacy::adapters::db::sql::seaorm::SeaormDriver;
use hextacy::adapters::queue::redis::RedisMessageQueue;
use hextacy::adapters::queue::redis::RedisPublisher;

pub type AuthenticationService = Authentication<
    SeaormDriver,
    UserAdapter,
    SessionAdapter,
    RedisPublisher,
>;

#[derive(Debug, Clone, State)]
pub struct AppState {
    #[env("DATABASE_URL")]
    #[load_async]
    pub repository: SeaormDriver,

    #[env(
        "RD_HOST",
        "RD_PORT" as u16,
        "RD_USER" as Option,
        "RD_PASSWORD" as Option,
    )]
    pub redis_q: RedisMessageQueue,
}

impl AuthenticationService {
    pub async fn init(state: &AppState) -> AuthenticationService {
        AuthenticationService::new(
            state.repository.clone(),
            UserAdapter,
            SessionAdapter,
            state
                .redis_q
                .publisher("my-channel")
                .await
                .expect("Could not create publisher"),
        )
    }
}
```

_Neat!_

For each field annotated with `env`, the `State` derive macro will attempt to call the type's associated `new` function, loading variables from `std::env` beforehand and passing them to the call. Luckily, both of these structs have them so we get an `AppState::load_repository_env` function and the same for `redis_q`. The `as` will attempt to parse the value of the env variable before passing it to `new`.

In the impl block for the service we set it up by calling it's `new` function, created from the `component` macro. All the components being passed satisfy the service's bounds. Here it's worth mentioning that the adapters are zero-sized, meaning they do not actually allocate any memory and are here simply to satisfy the bound restriction of the service, a sort of behaviour struct. The repository is cloned, which clones only the underlying reference to the connection pool and a publisher is created.

Finally, the main function.

```rust
#[tokio::main]
async fn main() {
    hextacy::env::load_from_file("path/to/.env").unwrap();

    let state = AppState::configure().await.unwrap();

    let (host, port) = (
        env::get_or_default("HOST", "127.0.0.1"),
        env::get_or_default("PORT", "3000"),
    );

        info!("Starting server on {addr}");

    let router = router(&state).await;

    axum::Server::bind(&addr.parse().unwrap())
        .serve(router.into_make_service())
        .await
        .expect("couldn't start server");
}

pub async fn router(state: &AppState) -> Router {
    use crate::controllers::http::auth::*;
    let auth_service = AuthenticationService::init(state).await;
    let router = Router::new()
        .route("/register", post(register))
        .route("/login", post(login));

    Router::new().nest("/auth", router).with_state(service)
}
```

And we have a working app! We haven't talked about how the files are set up, because this largely depends on preference and is ultimately arbitrary.

Next up, we'll ensure our app works by writing some tests.
