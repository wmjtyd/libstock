/// Get a timestamp whose format is `%Y%m%d`.
pub fn get_timestamp() -> String {
    use chrono::Local;

    let local_time = Local::now();
    local_time.format("%Y%m%d").to_string()
}
