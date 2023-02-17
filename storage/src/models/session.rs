use super::role::Role;
use super::user::User;
use alx_clients::oauth::OAuthProvider;
use chrono::NaiveDateTime;
use diesel::{
    deserialize::{self, FromSql},
    pg::{Pg, PgValue},
    serialize::{self, IsNull, Output, ToSql},
    sql_types::Text,
    AsExpression, FromSqlRow, Queryable,
};
use serde::{Deserialize, Serialize};
use std::io::Write;

#[derive(Debug, Clone, Deserialize, Serialize, FromSqlRow, AsExpression, PartialEq, Eq)]
#[diesel(sql_type = Text)]
pub enum AuthType {
    Native,
    OAuth(OAuthProvider),
}

impl ToSql<Text, Pg> for AuthType {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            AuthType::Native => out.write_all(b"native")?,
            AuthType::OAuth(provider) => match provider {
                OAuthProvider::Google => out.write_all(b"oauth:google")?,
                OAuthProvider::Github => out.write_all(b"oauth:github")?,
            },
        }
        Ok(IsNull::No)
    }
}

impl FromSql<Text, Pg> for AuthType {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"native" => Ok(AuthType::Native),
            b"oauth:google" => Ok(AuthType::OAuth(OAuthProvider::Google)),
            b"oauth:github" => Ok(AuthType::OAuth(OAuthProvider::Github)),
            _ => Err("Unrecognized Role variant".into()),
        }
    }
}

/// The repository session model
#[derive(Debug, Clone, Serialize, Deserialize, Queryable)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub username: String,
    pub email: String,
    pub phone: Option<String>,
    pub role: Role,
    pub csrf: String,
    pub oauth_token: Option<String>,
    pub auth_type: AuthType,
    #[serde(with = "ts_datetime")]
    pub created_at: NaiveDateTime,
    #[serde(with = "ts_datetime")]
    pub updated_at: NaiveDateTime,
    #[serde(with = "ts_datetime")]
    pub expires_at: NaiveDateTime,
}

impl Session {
    /// Check whether the session has an expiration time
    pub fn is_permanent(&self) -> bool {
        self.expires_at.timestamp() == NaiveDateTime::MAX.timestamp()
    }

    pub fn __mock(id: String, user: &User, csrf: String, permanent: bool) -> Self {
        Self {
            id,
            user_id: user.id.clone(),
            username: user.username.clone(),
            role: user.role.clone(),
            csrf,
            created_at: NaiveDateTime::from_timestamp_opt(chrono::Utc::now().timestamp(), 0)
                .unwrap(),
            updated_at: NaiveDateTime::from_timestamp_opt(chrono::Utc::now().timestamp(), 0)
                .unwrap(),
            expires_at: if permanent {
                NaiveDateTime::MAX
            } else {
                NaiveDateTime::from_timestamp_opt(chrono::Utc::now().timestamp(), 0).unwrap()
                    + chrono::Duration::minutes(30)
            },
            email: user.email.clone(),
            phone: user.phone.clone(),
            oauth_token: None,
            auth_type: AuthType::Native,
        }
    }
}

/// Serde utility for serializing `NaiveDateTime`s to timestamps and vice versa.
mod ts_datetime {
    use chrono::NaiveDateTime;
    use serde::{self, Deserialize, Deserializer, Serializer};
    // The signature of a serialize_with function must follow the pattern:
    //
    //    fn serialize<S>(&T, S) -> Result<S::Ok, S::Error>
    //    where
    //        S: Serializer
    //
    pub fn serialize<S>(date: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i64(date.timestamp())
    }

    // The signature of a deserialize_with function must follow the pattern:
    //
    //    fn deserialize<'de, D>(D) -> Result<T, D::Error>
    //    where
    //        D: Deserializer<'de>
    //
    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = i64::deserialize(deserializer)?;
        NaiveDateTime::from_timestamp_millis(millis)
            .ok_or(serde::de::Error::custom("Invalid timestamp"))
    }
}
