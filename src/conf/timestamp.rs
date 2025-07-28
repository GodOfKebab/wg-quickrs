use chrono::{DateTime, Duration, SecondsFormat, TimeDelta, Utc};

pub fn get_now_timestamp_formatted() -> String {
    return Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
}
pub fn get_future_timestamp_formatted(delay: Duration) -> String {
    return (Utc::now() + delay).to_rfc3339_opts(SecondsFormat::Secs, true);
}

pub fn get_duration_since_formatted(timestamp: String) -> TimeDelta {
    let timestamp_utc = DateTime::parse_from_rfc3339(&*timestamp)
        .expect("Invalid RFC3339 time")
        .with_timezone(&Utc);

    return Utc::now().signed_duration_since(timestamp_utc);
}



