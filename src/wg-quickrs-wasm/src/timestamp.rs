use chrono::{DateTime, Duration, SecondsFormat, TimeDelta, Utc};

pub fn get_now_timestamp_formatted() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}
pub fn get_future_timestamp_formatted(delay: Duration) -> String {
    (Utc::now() + delay).to_rfc3339_opts(SecondsFormat::Secs, true)
}

pub fn get_duration_since_formatted(timestamp: &String) -> Option<TimeDelta> {
    let timestamp_utc = match DateTime::parse_from_rfc3339(timestamp) {
        Ok(t) => t.with_timezone(&Utc),
        Err(_) => return None,
    };
    Some(Utc::now().signed_duration_since(timestamp_utc))
}
