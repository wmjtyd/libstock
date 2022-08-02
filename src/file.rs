//! The file writer daemon, reader and identifier generator of `libstock`.

#[deprecated(since = "0.4.0", note = "We don't use this identifier anymore.")]
pub mod ident;

pub mod reader;
pub mod timestamp;
pub mod writer;

mod datadir;

#[cfg(test)]
mod tests {
    use super::reader::FileReader;
    use super::writer::{DataEntry, DataWriter};

    #[tokio::test]
    async fn test_read_write() {
        {
            // Add log to catch errors.
            tracing_subscriber::fmt::init();

            const CONTENT_A: &[u8] = b"Hello, world!";
            const CONTENT_B: &[u8] = b"<OwO>";

            let filename = uuid::Uuid::new_v4().to_string();

            //////////////////
            // Writer Part ///
            //////////////////

            // The writer to write today's content;
            let mut writer = DataWriter::new();
            let writer_thread = writer.start().await.expect("failed to spawn writer");

            writer
                .add(DataEntry {
                    filename: filename.to_string(),
                    data: CONTENT_A.to_vec(),
                })
                .expect("failed to add CONTENT_A");

            writer
                .add(DataEntry {
                    filename: filename.to_string(),
                    data: CONTENT_B.to_vec(),
                })
                .expect("failed to add CONTENT_B");

            writer.stop().expect("failed to stop writer");
            writer_thread
                .await
                .expect("failed to wait writer thread to stop");

            //////////////////
            // Reader Part ///
            //////////////////

            // The reader to read today's content.
            let mut reader = FileReader::new(filename.to_string(), 0)
                .expect("failed to start the reader to read the written data.");

            assert_eq!(reader.next().expect("unexpected end (1)"), CONTENT_A);
            assert_eq!(reader.next().expect("unexpected end (2)"), CONTENT_B);
            assert!(reader.next().is_none());
        }
    }
}
