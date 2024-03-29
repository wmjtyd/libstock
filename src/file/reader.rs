//! Read the specified file and return the content stream.

use std::fs::File;
use std::io::Read;

use chrono::{Duration, Local};

use crate::file::datadir::get_ident_path;
use crate::file::timestamp::fmt_timestamp;

pub struct FileReader {
    file: File,
}

impl FileReader {
    pub fn new(filename: String, day: i64) -> std::io::Result<FileReader> {
        let time = Local::now() - Duration::days(day);
        let timestamp = fmt_timestamp(&time);

        let path = get_ident_path(&timestamp, &filename);
        tracing::info!("Creating a writer to read {path}", path = path.display());

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
