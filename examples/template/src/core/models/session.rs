use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// In seconds, 24 hours
pub const SESSION_DURATION: i64 = 60 * 60 * 24;

/// Internal session used by the server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub csrf: Uuid,
    #[serde(with = "ts_datetime")]
    pub created_at: NaiveDateTime,
    #[serde(with = "ts_datetime")]
    pub updated_at: NaiveDateTime,
    #[serde(with = "ts_datetime")]
    pub expires_at: NaiveDateTime,
}

impl Session {
    pub fn new(user_id: Uuid, expires: bool) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            csrf: Uuid::new_v4(),
            created_at: NaiveDateTime::from_timestamp_opt(Utc::now().timestamp(), 0).unwrap(),
            updated_at: NaiveDateTime::from_timestamp_opt(Utc::now().timestamp(), 0).unwrap(),
            expires_at: if expires {
                NaiveDateTime::from_timestamp_opt(Utc::now().timestamp() + SESSION_DURATION, 0)
                    .unwrap()
            } else {
                NaiveDateTime::MAX
            },
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

impl From<crate::db::entities::sessions::Model> for Session {
    fn from(
        crate::db::entities::sessions::Model {
            id,
            user_id,
            csrf,
            created_at,
            updated_at,
            expires_at,
        }: crate::db::entities::sessions::Model,
    ) -> Self {
        Self {
            id,
            user_id,
            csrf,
            created_at: created_at.naive_utc(),
            updated_at: updated_at.naive_utc(),
            expires_at: expires_at.naive_utc(),
        }
    }
}

impl From<Session> for crate::db::entities::sessions::ActiveModel {
    fn from(
        Session {
            id,
            user_id,
            csrf,
            created_at,
            updated_at,
            expires_at,
        }: Session,
    ) -> crate::db::entities::sessions::ActiveModel {
        crate::db::entities::sessions::ActiveModel {
            id: sea_orm::Set(id),
            user_id: sea_orm::Set(user_id),
            csrf: sea_orm::Set(csrf),
            created_at: sea_orm::Set(created_at.and_utc().fixed_offset()),
            updated_at: sea_orm::Set(updated_at.and_utc().fixed_offset()),
            expires_at: sea_orm::Set(expires_at.and_utc().fixed_offset()),
        }
    }
}
