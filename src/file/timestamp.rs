use chrono::{DateTime, Local};

/// Get the timestamp whose format is `%Y%m%d` of current time.
pub fn get_timestamp() -> String {
    let local_time = Local::now();

    fmt_timestamp(&local_time)
}

/// Format a timestamp to `%Y%m%d`.
/// 
/// # Example
/// 
/// ```
/// use wmjtyd_libstock::file::timestamp::fmt_timestamp;
/// use chrono::{DateTime, Local};
/// 
/// let local_time = Local::now();
/// 
/// let lf = local_time.format("%Y%m%d").to_string();
/// let rf = fmt_timestamp(&local_time);
/// 
/// assert_eq!(lf, rf);
/// ```
pub fn fmt_timestamp(t: &DateTime<Local>) -> String {
    t.format("%Y%m%d").to_string()
}
