use std::path::PathBuf;
use concat_string::concat_string;

/// Get the data directory.
/// 
/// Currently, the data directory is `./record`.
pub fn get_data_directory() -> PathBuf {
    let mut path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    path.push("record");
    path
}

/// Get the exact filename to write to.
pub fn get_path_to_write(identifier: &str) -> PathBuf {
    let mut path = get_data_directory();
    path.push(concat_string!(identifier, ".csv"));

    path
}
