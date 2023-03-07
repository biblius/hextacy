use std::{cmp::Ordering, io::Write};

use diesel::{
    deserialize::{self, FromSql},
    pg::{Pg, PgValue},
    serialize::{self, IsNull, Output, ToSql},
    sql_types::Text,
    AsExpression, FromSqlRow,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, FromSqlRow, AsExpression, PartialEq, Eq)]
#[diesel(sql_type = Text)]
pub enum Role {
    #[serde(rename = "admin")]
    Admin,
    #[serde(rename = "user")]
    User,
}

impl Default for Role {
    fn default() -> Self {
        Self::User
    }
}

impl PartialOrd for Role {
    fn partial_cmp(&self, other: &Role) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Role {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            Role::Admin => match other {
                Role::Admin => Ordering::Equal,
                Role::User => Ordering::Greater,
            },
            Role::User => match other {
                Role::Admin => Ordering::Less,
                Role::User => Ordering::Equal,
            },
        }
    }
}

impl ToSql<Text, Pg> for Role {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            Role::Admin => out.write_all(b"admin")?,
            Role::User => out.write_all(b"user")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<Text, Pg> for Role {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"admin" => Ok(Role::Admin),
            b"user" => Ok(Role::User),
            _ => Err("Unrecognized Role variant".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn enum_serde() {
        #[derive(Debug, Serialize, Deserialize)]
        struct Test {
            pub role: Role,
        }

        let role = Test { role: Role::Admin };
        let s = serde_json::to_string(&role).unwrap();

        assert_eq!("{\"role\":\"admin\"}", s);

        let d = serde_json::from_str::<Test>(&s).unwrap();

        assert!(matches!(d.role, Role::Admin))
    }
}
