use hextacy::adapters::cache::redis::RedisDriver;
use hextacy::adapters::db::sql::seaorm::SeaormDriver;
use hextacy::adapters::queue::amqp::AmqpDriver;
use hextacy::adapters::queue::redis::RedisMessageQueue;
use hextacy::queue::{Consumer, Producer};
use hextacy::State;
use serde_json::json;

use crate::db::adapters::user::UserAdapter;

use super::queue::{HelloWorld, HelloWorldHandler, RegisterUser};

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
        "RD_DATABASE" as i64,
    )]
    pub cache: RedisDriver,

    #[env(
        "RD_HOST",
        "RD_PORT" as u16,
        "RD_USER" as Option,
        "RD_PASSWORD" as Option,
    )]
    pub redis_q: RedisMessageQueue,
    /*
    #[env("AMQP_ADDRESS")]
    #[load_async]
    pub amqp: AmqpDriver, */
}

impl AppState {
    pub async fn init() -> Self {
        let repository = AppState::load_repository_env()
            .await
            .expect("Could not load repository");

        let cache = AppState::load_cache_env().expect("Could not load cache");

        /*         let amqp = AppState::load_amqp_env()
                   .await
                   .expect("Could not load amqp");
        */
        let redis_q = AppState::load_redis_q_env().expect("fuck");

        let mut publisher = redis_q.publisher("my-queue").await.unwrap();
        let consumer = redis_q.consumer("my-queue").await.unwrap();
        consumer.start(HelloWorldHandler::new(repository.clone(), UserAdapter));

        publisher
            .publish(HelloWorld::World(RegisterUser {
                username: "hello".to_string(),
                password: "world".to_string(),
            }))
            .await
            .unwrap();
        publisher
            .publish(json!({
                "username": "hello",
                "password": "world"
            }))
            .await
            .unwrap();
        publisher
            .publish(json!({
                "username": "hello",
                "password": "world"
            }))
            .await
            .unwrap();

        Self {
            repository,
            cache,
            //            amqp,
            redis_q,
        }
    }
}
