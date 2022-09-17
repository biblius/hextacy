use config::{get_from_env, load_from_file, set};

pub fn set_env_vars() {
    set("DB_URL", "localhost:postgres:lmao");
    set("SOME_VAR", "SOME_VALUE");
    let mut from_env = get_from_env(vec!["DB_URL", "SOME_VAR", "NOT_EXISTS"]);

    assert_eq!(from_env.pop().unwrap(), None);
    assert_eq!(from_env.pop().unwrap(), Some("SOME_VALUE".to_string()));
    assert_eq!(
        from_env.pop().unwrap(),
        Some("localhost:postgres:lmao".to_string())
    );
}

pub fn load_env() {
    load_from_file("./tests/.env", config::ConfigFormat::DotEnv).unwrap();
}
