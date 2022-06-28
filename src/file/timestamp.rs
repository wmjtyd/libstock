use chrono::{DateTime, Local};

/// Get a timestamp whose format is `%Y%m%d`.
pub fn get_timestamp() -> String {
    let local_time = Local::now();
    local_time.format("%Y%m%d").to_string()
}

/// Format a timestamp to `%Y%m%d`.
pub fn fmt_timestamp(t: DateTime<Local>) -> String {
    t.format("%Y%m%d").to_string()
}
