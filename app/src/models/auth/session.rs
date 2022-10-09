use super::super::{role::Role, schema::sessions};
use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Queryable)]
pub struct Session {
    id: String,
    user_id: String,
    session_token: String,
    csrf_token: String,
    user_role: Role,
    username: String,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
    expires_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Insertable)]
#[diesel(table_name = sessions)]
pub struct NewSession<'a> {
    user_id: &'a str,
    session_token: &'a str,
    csrf_token: &'a str,
    user_role: Role,
    username: &'a str,
}
