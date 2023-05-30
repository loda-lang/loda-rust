use chrono::prelude::*;

/// Returns a human readable timestamp in UTC.
/// 
/// Example: `1984-12-28T23:35:56Z`
pub fn human_readable_utc_timestamp() -> String {
    let datetime: DateTime<Utc> = Utc::now();
    datetime.to_rfc3339_opts(SecondsFormat::Secs, true).to_string()
}
