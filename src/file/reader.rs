//! Read the specified file and return the content stream.

use chrono::{Duration, Local};
use std::{fs::File, io::Read};

use crate::file::{datadir::get_ident_path, ident::get_ident, timestamp::fmt_timestamp};

pub struct FileReader {
    file: File,
}

impl FileReader {
    pub fn new(filename: String, day: i64) -> std::io::Result<FileReader> {
        let time = Local::now() - Duration::days(day);
        let timestamp = fmt_timestamp(&time);
        tracing::info!("Creating a writer to read {filename}, timestamp {timestamp}");
        
        let identifier = get_ident(&filename, &timestamp);
        let path = get_ident_path(&identifier);

        File::open(path).map(|file| FileReader { file })
    }
}

// http utp utp:quic
impl Iterator for FileReader {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut data_len_section = [0u8; 2];
        if let Ok(()) = self.file.read_exact(&mut data_len_section) {
            let data_len = u16::from_be_bytes(data_len_section) as usize;
            let mut data = vec![0u8; data_len];

            if let Err(e) = self.file.read_exact(&mut data) {
                tracing::error!("Failed to read the complete data: {e}. Returning None.");
                None
            } else {
                Some(data)
            }
        } else {
            None
        }
    }
}
