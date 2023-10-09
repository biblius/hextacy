use chrono::{DateTime, Utc};
use hextacy::exports::uuid::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(username: String, password: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            username,
            password,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

impl From<crate::db::entities::users::Model> for User {
    fn from(
        crate::db::entities::users::Model {
            id,
            username,
            password,
            created_at,
            updated_at,
        }: crate::db::entities::users::Model,
    ) -> Self {
        Self {
            id,
            username,
            password,
            created_at: created_at.into(),
            updated_at: updated_at.into(),
        }
    }
}

impl From<User> for crate::db::entities::users::ActiveModel {
    fn from(
        User {
            id,
            username,
            password,
            created_at,
            updated_at,
        }: User,
    ) -> crate::db::entities::users::ActiveModel {
        crate::db::entities::users::ActiveModel {
            id: sea_orm::Set(id),
            username: sea_orm::Set(username),
            password: sea_orm::Set(password),
            created_at: sea_orm::Set(created_at.into()),
            updated_at: sea_orm::Set(updated_at.into()),
        }
    }
}
