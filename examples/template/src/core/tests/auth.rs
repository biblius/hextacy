#[suitest::suite(auth_service_integration_tests)]
pub mod integration_tests {

    use suitest::*;

    use std::time::Duration;

    use hextacy::Driver;

    use crate::{
        cache::adapters::RedisAdapter,
        config::state::{AppState, AuthenticationService},
        core::models::user::User,
        db::{
            adapters::{session::SessionAdapter, user::UserAdapter},
            driver::SeaormDriver,
            entities::sessions::Model as SessionModel,
            entities::users::{ActiveModel as ActiveUserModel, Model as UserModel},
        },
    };

    #[before_all]
    async fn before_all() -> (SeaormDriver, AuthenticationService) {
        hextacy::env::load_from_file(".env").expect("couldn't load env");
        let app = AppState::load().await.unwrap();
        (
            app.repository.clone(),
            AuthenticationService::new(
                app.repository.clone(),
                app.cache,
                UserAdapter,
                SessionAdapter,
                RedisAdapter,
                app.redis_q.publisher("my-queue").await.unwrap(),
            ),
        )
    }

    #[before_each]
    async fn before_each(driver: SeaormDriver) -> User {
        let conn = driver.connect().await.unwrap();
        let user: ActiveUserModel = User::new("foomao".to_string(), "barofl".to_string()).into();
        let user: User = driver.insert(&conn, user).await.unwrap();
        user
    }

    #[after_each]
    async fn after_each(user: User, driver: SeaormDriver) {
        let conn = driver.connect().await.unwrap();
        let num = driver
            .delete::<UserModel, _, _, _>(&conn, user.id)
            .await
            .unwrap();
        assert_eq!(num, 1);
    }

    #[cleanup]
    async fn failsafe(user: User, driver: SeaormDriver) {
        let conn = driver.connect().await.unwrap();
        driver
            .delete::<UserModel, _, _, _>(&conn, user.id)
            .await
            .unwrap();
    }

    #[after_all]
    async fn after_all() {}

    #[test]
    async fn some_test() {
        tokio::time::sleep(Duration::from_secs(1)).await;
        // panic!("foo");
    }

    #[test]
    async fn other_test() {
        tokio::time::sleep(Duration::from_secs(1)).await;
        // panic!("bar");
    }

    #[test]
    async fn registration(driver: SeaormDriver, service: AuthenticationService) {
        let (user, session) = service.register("fooser", "passbar").await.unwrap();

        let conn = driver.connect().await.unwrap();
        let user = driver
            .get_by_id::<User, UserModel, _, _, _>(&conn, user.id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(user.id, session.user_id);

        let count = driver
            .delete::<UserModel, _, _, _>(&conn, user.id)
            .await
            .unwrap();
        assert_eq!(count, 1);

        let count = driver
            .delete::<SessionModel, _, _, _>(&conn, user.id)
            .await
            .unwrap();
        assert_eq!(count, 0);
    }
}
