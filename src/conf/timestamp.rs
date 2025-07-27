use chrono::{Duration, NaiveDateTime, TimeDelta, Utc};

pub(crate) const DATETIME_FORMAT: &str = "%Y-%m-%dT%H:%M:%S%.3fZ";

pub fn get_now_timestamp_formatted() -> String {
    return Utc::now().format(DATETIME_FORMAT).to_string();
}
pub fn get_future_timestamp_formatted(delay: Duration) -> String {
    return (Utc::now() + delay).format(DATETIME_FORMAT).to_string();
}

pub fn get_duration_since_formatted(timestamp: String) -> TimeDelta {
    let naive_datetime = NaiveDateTime::parse_from_str(&*timestamp, DATETIME_FORMAT).unwrap();
    return naive_datetime - Utc::now().naive_utc();
}



