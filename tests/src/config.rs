use config::{get_multiple, load_from_file, set};
use std::env;
use tracing::info;

pub fn set_env_vars() {
    info!("\n========== TEST - SET ENV VARS ==========\n");
    set("DB_URL", "localhost:postgres:lmao");
    set("SOME_VAR", "SOME_VALUE");
    let mut from_env = get_multiple(vec!["DB_URL", "SOME_VAR"]);

    assert_eq!(from_env.pop(), Some("SOME_VALUE".to_string()));
    assert_eq!(from_env.pop(), Some("localhost:postgres:lmao".to_string()));
}

pub fn load_from_dot_env(path: &str) {
    info!("\n========== TEST - LOAD DOT ENV ==========\n");
    load_from_file(path, config::ConfigFormat::DotEnv).unwrap();
    let db_url = env::var("POSTGRES_URL").unwrap();
    let max_conns = env::var("PG_POOL_SIZE").unwrap();
    assert_eq!(
        db_url,
        "postgresql://postgres:postgres@localhost:5432/myco_test_db"
    );
    assert_eq!(max_conns, "8")
}
