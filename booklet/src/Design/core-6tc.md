# Core ⬡

If we take a look at the last impl block in the previous section, we can notice a pattern. We see the 2 repositories pretty much have the same driver bounds and everything has our beloved `Send` bound. If we were to add more, the pattern would repeat. Fortunately, rust provides us with excellent tooling to eliminate hand written repetition - macros! You know, those things you use to annotate your structs to print them to the terminal and stuff.

```rust
use hextacy::{component, transaction};

#[component(
    use D as driver,
    use UserRepo, SessionRepo
)]
#[derive(Debug, Clone)]
pub struct Authentication {}

#[component(
    use D:Atomic for
        UR: UserRepository,
        SR: SessionRepository,
)]
impl Authentication {
    pub async fn register(&self, username: &str, password: &str) -> AppResult<Session> {
        let mut conn = self.driver.connect().await?;

        match self.user_repo.get_by_username(&mut conn, username).await {
            Ok(None) => {}
            Ok(Some(_)) => return Err(AuthenticationError::UsernameTaken.into()),
            Err(e) => return Err(e.into()),
        };

        let hashed = hextacy::crypto::bcrypt_hash(password, 10)?;

        let session: Session = transaction!(
            conn: D => {
                let user = self.user_repo.create(&mut conn, username, &hashed).await?;
                let session = self.session_repo.create(&mut conn, &user, true).await?;
                Ok(session)
            }
        )?;

        Ok(session)
    }
}
```

Ain't it neat?

Now that we've seen how a decoupled service would look like in 'vanilla' rust, we can dive in the `component` and `transaction` macros. The macros create the exact same code we've had to create by hand in the last part of the last section.

The first invocation of the `component` macro on the struct definition creates a completely generic struct whose fields are exactly the same as the hand written implementation (PascalCase gets transformed into snake_case). For convenience, it also receives an associated `new` function.

The second invocation takes the annotated impl block and 'injects' all the necessary generics and binds them to their respective types. This macro gives us a simple and concise way of specifying the repository components this service will use.

The `transaction` macro allows us to easily write atomic queries without having to match the result every time. It takes in a connection (the variable `conn` in our case) and uses it to start a transaction before running whatever is inside the block. The block must return a `Result<T>` to be usable in the macro. Because we have the `AppResult` type, which is just `Result<T, AppError>`, we can use `thiserror` to easily create the `From` implementations for our global `AppError` and question everything where applicable. If any operations fail, the closure returns an error and the transaction is aborted.

Another cool thing about the `component` macro is that it can be used on structs with existing fields and impl blocks. To demonstrate we'll add our final requirement, the message broker.

```rust
use hextacy::{component, transaction, queue::Publisher}; //

#[derive(Debug, Serialize)] //
pub struct UserRegisteredEvent {
    id: Uuid,
    username: String,
}

#[component(
    use D as driver,
    use UserRepo, SessionRepo, Publisher //
)]
#[derive(Debug, Clone)]
pub struct Authentication<Existing> { //
    e: Existing // Just to demonstrate
    foo: usize, //
}

#[component(
    use D:Atomic for
        UR: UserRepository,
        SR: SessionRepository,
)]
impl<P, E> Authentication<P, E> //
where
    P: Producer, //
    E: Debug // Ordering matters here, existing stuff goes after the macro stuff
{
    pub async fn register(&self, username: &str, password: &str) -> AppResult<Session> {
        let mut conn = self.driver.connect().await?;

        match self.user_repo.get_by_username(&mut conn, username).await {
            Ok(None) => {}
            Ok(Some(_)) => return Err(AuthenticationError::UsernameTaken.into()),
            Err(e) => return Err(e.into()),
        };

        let hashed = hextacy::crypto::bcrypt_hash(password, 10)?;

        let session: Session = transaction!(
            conn: D => {
                let user = self.user_repo.create(&mut conn, username, &hashed).await?;
                let session = self.session_repo.create(&mut conn, &user, true).await?;
                self.publisher //
                    .publish(UserRegisteredEvent {
                      id: user.id,
                      username: user.username,
                })
                .await?;
                Ok(session)
            }
        )?;

        Ok(session)
    }
}
```

We've added some existing generics to the struct and it still works! The ordering is important though, so we have to keep in mind the existing generics, i.e. generics outside the `component` macro are always the last items in the struct.

We've also added a publisher through the macro (we could've explicitly added it, but it's more concise with `component`) and in the impl block we've bound it to `Producer` which enables us to publish any structs that can be serialized. Do note if the publishing fails, neither of the last 2 state changes are applied. The service doesn't know where it'll be publishing, but that is not its concern and is up to the implementation.

And that would be the end of our core logic - we've met the extreme requirements posed on us and designed a service with only the business, albeit not a very complex one. Most importantly, we haven't leaked any implementation details into the service. Instead, we've bounded its generic parameters to contracts which concrete instances must fulfil in order for the service to be constructed. Traits rule!

Now we actually need to get the thing running, which is what we'll be exploring in the next section.
