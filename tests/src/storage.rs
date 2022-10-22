use crate::schema::{simple_models, test_users};
use diesel::{
    query_dsl::methods::{FilterDsl, LimitDsl, OrderDsl, SelectDsl},
    ExpressionMethods, Insertable, Queryable, RunQueryDsl,
};
use infrastructure::clients::{mongo::MongoSync, postgres::Postgres, redis::Redis};
use mongodb::bson::doc;
use tracing::{info, trace};

pub fn establish_pg_connection() {
    info!("\n========== TEST - ESTABLISH PG CONNECTION (POOL + DIRECT) ==========\n");

    let pool = Postgres::default();
    let conn = pool.connect();
    assert!(matches!(conn, Ok(_)));
    let dir_conn = Postgres::connect_direct();
    assert!(matches!(dir_conn, Ok(_)));
}

pub fn establish_rd_connection() {
    info!("\n========== TEST - ESTABLISH RD CONNECTION (POOL + DIRECT) ==========\n");

    let pool = Redis::new();
    let conn = pool.connect();
    assert!(matches!(conn, Ok(_)));
    let dir_conn = Redis::connect_direct();
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

    let mut conn = Postgres::connect_direct().unwrap();

    let user = NewTestUser {
        username: "i am user".to_string(),
        password: "super secret".to_string(),
    };

    let data = NewSimpleModel {
        some_param: "param".to_string(),
        other_param: 12,
    };

    let result = conn.build_transaction().deferrable().run(|conn| {
        let mut user: Vec<TestUser> = diesel::insert_into(test_users::table)
            .values(user)
            .get_results(conn)
            .expect("Couldn't insert user");

        trace!("{:?}", user);

        let mut simple = match diesel::insert_into(simple_models::table)
            .values(data)
            .get_results::<SimpleModel>(conn)
        {
            Ok(simple) => simple,
            Err(_) => return Err(diesel::result::Error::RollbackTransaction),
        };

        trace!("{:?}", simple);
        Ok((user.pop().unwrap(), simple.pop().unwrap()))
    });
    assert!(matches!(result, Ok(_)))
}

pub fn pg_transaction_fail() {
    info!("\n========== TEST - PG INSERT WITH TRANSACTION FAIL ==========\n");
    let mut conn = Postgres::connect_direct().unwrap();

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

    let result = conn.build_transaction().deferrable().run(|conn| {
        let user: Vec<TestUser> = diesel::insert_into(test_users::table)
            .values(user)
            .get_results(conn)
            .expect("Couldn't insert user");

        trace!("User that should be rolled back : {:?}", user);

        // Error here after inserting the user to see if the transaction rolls back
        match diesel::insert_into(simple_models::table)
            .values(data)
            .get_results::<SimpleModel>(conn)
        {
            Ok(_) => return Err(diesel::result::Error::RollbackTransaction),
            Err(_) => return Err(diesel::result::Error::RollbackTransaction),
        };
        #[allow(unreachable_code)]
        Ok(user)
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
