use mongodb::bson::doc;
use r2d2_redis::redis::ConnectionAddr;
use storage::mongo::MongoSync;
use storage::postgres::Pg;
use storage::redis::{connection_info_default, Rd};
use tracing::{info, trace};

pub fn establish_pg_connection() {
    info!("========== TEST - ESTABLISH PG CONNECTION (POOL + DIRECT) ==========");

    let pool = Pg::default();
    let conn = pool.connect();
    assert!(matches!(conn, Ok(_)));
    let dir_conn = Pg::connect_direct();
    assert!(matches!(dir_conn, Ok(_)));
}

pub fn rd_default_conn_info() {
    info!("========== TEST - RD DEFAULT CONNECTION INFO ==========");

    let ci = connection_info_default();
    assert_eq!(
        *ci.addr,
        ConnectionAddr::Tcp(String::from("127.0.0.1"), 6379)
    );
    assert_eq!(ci.db, 0);
    assert_eq!(ci.username, None);
    assert_eq!(ci.passwd, None);
}

pub fn mongo_default_client_options() {
    info!("========== TEST - MONGO DEFAULT CLIENT OPTIONS ==========");
}

pub fn establish_rd_connection() {
    info!("========== TEST - ESTABLISH RD CONNECTION (POOL + DIRECT) ==========");

    let pool = Rd::new();
    let conn = pool.connect();
    assert!(matches!(conn, Ok(_)));
    let dir_conn = Rd::connect_direct();
    assert!(matches!(dir_conn, Ok(_)));
}

pub fn mongo_insert_with_transaction() {
    info!("========== TEST - MONGO INSERT WITH TRANSACTION ==========");

    let mongo = MongoSync::new();

    let mut session = mongo.client.start_session(None).unwrap();

    session.start_transaction(None).unwrap();

    let db = mongo.client.default_database().unwrap();

    let result = db
        .collection("test")
        .insert_many(
            vec![
                doc! {"some_param": "7", "other_param": "roflolmeo"},
                doc! {"some_param": "420", "other_param": "the otherest parameter"},
            ],
            None,
        )
        .unwrap();

    trace!("{:?}", result);

    session.commit_transaction().unwrap();
}
