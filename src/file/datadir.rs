use concat_string::concat_string;
use std::path::PathBuf;

/// Get the data directory.
///
/// Currently, the data directory is `./record/<timestamp>`.
pub fn get_data_directory(timestamp: &str) -> PathBuf {
    let mut path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    path.push("record");
    path.push(timestamp);
    path
}

/// Get the exact filename to write to.
pub fn get_ident_path(timestamp: &str, identifier: &str) -> PathBuf {
    let mut path = get_data_directory(timestamp);
    path.push(concat_string!(identifier, ".csv"));

    path
}
