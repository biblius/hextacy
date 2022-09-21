use crate::schema::{simple_models, test_users};
use diesel::{
    query_dsl::methods::{FilterDsl, LimitDsl, OrderDsl, SelectDsl},
    ExpressionMethods, Insertable, Queryable, RunQueryDsl,
};
use mongodb::bson::doc;
use r2d2_redis::redis::ConnectionAddr;
use storage::{
    mongo::MongoSync,
    postgres::{Pg, SqlModel},
    redis::{connection_info_default, Rd},
};

use tracing::{info, trace};

pub fn establish_pg_connection() {
    info!("\n========== TEST - ESTABLISH PG CONNECTION (POOL + DIRECT) ==========\n");

    let pool = Pg::default();
    let conn = pool.connect();
    assert!(matches!(conn, Ok(_)));
    let dir_conn = Pg::connect_direct();
    assert!(matches!(dir_conn, Ok(_)));
}

pub fn rd_default_conn_info() {
    info!("\n========== TEST - RD DEFAULT CONNECTION INFO ==========\n");

    let ci = connection_info_default();
    assert_eq!(
        *ci.addr,
        ConnectionAddr::Tcp(String::from("localhost"), 6379)
    );
    assert_eq!(ci.db, 0);
    assert_eq!(ci.username, None);
    assert_eq!(ci.passwd, None);
}

pub fn establish_rd_connection() {
    info!("\n========== TEST - ESTABLISH RD CONNECTION (POOL + DIRECT) ==========\n");

    let pool = Rd::new();
    let conn = pool.connect();
    assert!(matches!(conn, Ok(_)));
    let dir_conn = Rd::connect_direct();
    assert!(matches!(dir_conn, Ok(_)));
}

pub fn mongo_insert_with_transaction() {
    info!("\n========== TEST - MONGO INSERT WITH TRANSACTION ==========\n");

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

pub fn pg_transaction() {
    info!("\n========== TEST - PG INSERT WITH TRANSACTION SUCCESS ==========\n");
    let mut conn = Pg::connect_direct().unwrap();
    let user = NewTestUser {
        username: "i am user".to_string(),
        password: "super secret".to_string(),
    };
    let data = NewSimpleModel {
        some_param: "param".to_string(),
        other_param: 12,
    };
    let input: Vec<Box<dyn SqlModel>> = vec![Box::new(user), Box::new(data)];
    let result = Pg::transaction(input, &mut conn, |input, conn| {
        let mut user: Vec<TestUser> = diesel::insert_into(test_users::table)
            .values(input[0].as_any().downcast_ref::<NewTestUser>().unwrap())
            .get_results(conn)
            .expect("Couldn't insert user");

        trace!("{:?}", user);

        let mut simple: Vec<SimpleModel> = diesel::insert_into(simple_models::table)
            .values(input[1].as_any().downcast_ref::<NewSimpleModel>().unwrap())
            .get_results(conn)
            .expect("Couldn't insert simple model");

        trace!("{:?}", simple);
        Ok((user.pop().unwrap(), simple.pop().unwrap()))
    });
    assert!(matches!(result, Ok(_)))
}

pub fn pg_transaction_fail() {
    info!("\n========== TEST - PG INSERT WITH TRANSACTION FAIL ==========\n");
    let mut conn = Pg::connect_direct().unwrap();

    let user = NewTestUser {
        username: "i am user".to_string(),
        password: "super secret".to_string(),
    };

    let data = NewSimpleModel {
        some_param: "param".to_string(),
        other_param: 12,
    };

    // Grab the highest id of the existing user, after the transaction rolls back we check if there
    // exists an entry with an id equal to this one incremented by 1
    let highest_id: i32 = test_users::table
        .select(test_users::id)
        .order(test_users::id.desc())
        .limit(1)
        .load(&mut conn)
        .expect("Couldn't load user")
        .pop()
        .unwrap();

    trace!("Current highest id: {}", highest_id);

    let input: Vec<Box<dyn SqlModel>> = vec![Box::new(user), Box::new(data)];
    let result = Pg::transaction(input, &mut conn, |input, conn| {
        let new_user = input[0].as_any().downcast_ref::<NewTestUser>().unwrap();

        // This will downcast to the wrong type resulting in an error
        let new_simple = input[1].as_any().downcast_ref::<SimpleModel>();

        let user: Vec<TestUser> = diesel::insert_into(test_users::table)
            .values(new_user)
            .get_results(conn)
            .expect("Couldn't insert user");

        trace!("User that should be rolled back : {:?}", user);

        // Error here after inserting the user to see if the transaction rolls back
        match diesel::insert_into(simple_models::table)
            .values(new_simple)
            .get_results::<SimpleModel>(conn)
        {
            Ok(simple) => Ok((user, simple)),
            Err(_) => Err(diesel::result::Error::RollbackTransaction.into()),
        }
    });

    assert!(matches!(result, Err(_)));

    trace!("Searching for : {}", highest_id + 1);

    // Check if it was rolled back
    let non_existent = test_users::table
        .filter(test_users::id.eq(highest_id + 1))
        .load::<TestUser>(&mut conn)
        .unwrap();

    trace!("Found {} users, success!", non_existent.len());

    assert!(non_existent.is_empty());
}

#[derive(Debug, Queryable)]
#[allow(dead_code)]
struct TestUser {
    id: i32,
    username: String,
    password: String,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = test_users)]
struct NewTestUser {
    username: String,
    password: String,
}

#[derive(Debug, Insertable, Queryable)]
struct SimpleModel {
    id: i32,
    some_param: String,
    other_param: i32,
}
#[derive(Debug, Insertable)]
#[diesel(table_name = simple_models)]
struct NewSimpleModel {
    some_param: String,
    other_param: i32,
}

impl SqlModel for TestUser {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
impl SqlModel for NewTestUser {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
impl SqlModel for SimpleModel {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
impl SqlModel for NewSimpleModel {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
