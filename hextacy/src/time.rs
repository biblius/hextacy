/// Get a date time `s` seconds in the future
pub fn seconds_from_now(s: i64) -> chrono::NaiveDateTime {
    (chrono::Utc::now() + chrono::Duration::seconds(s)).naive_utc()
}

/// Get a timestamp from the current UTC time
pub fn now() -> i64 {
    chrono::Utc::now().timestamp()
}

pub fn datetime_now() -> chrono::NaiveDateTime {
    chrono::Utc::now().naive_utc()
}

pub fn date_now() -> chrono::NaiveDate {
    chrono::Utc::now().date_naive()
}
