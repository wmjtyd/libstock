//! The writer daemon to write data and place file automatically
//! without worrying about managing the path.

use std::fmt::Display;

use flume::{Receiver, Sender};
use tokio::{fs::OpenOptions, task::JoinHandle};
use tracing::Instrument;
use uuid::Uuid;

use crate::flag::BinaryFlag;

use super::{
    datadir::{get_data_directory, get_ident_path},
    ident::get_ident,
    timestamp::get_timestamp,
};

/// A owned data entry to send to a [`DataWriter`].
///
/// # Example
///
/// ```
/// use wmjtyd_libstock::file::writer::DataEntry;
///
/// let de = DataEntry {
///    filename: "test".to_string(),
///    data: b"OwO".to_vec(),
/// };
/// let de_clone = de.clone();
///
/// assert_eq!(de, de_clone);
/// ```
#[derive(Clone, Hash, PartialEq, Debug)]
pub struct DataEntry {
    /// The file name to write as.
    pub filename: String,
    /// The data to write.
    pub data: Vec<u8>,
}

impl Display for DataEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({} bytes)", self.filename, self.data.len())
    }
}

/// The writer daemon to write data and place file automatically,
/// without worrying about managing the path; and asynchronoly,
/// with a synchoronous `.add()` API.
///
/// # Example
///
/// ```
/// use wmjtyd_libstock::file::writer::{DataWriter, DataEntry};
///
/// let mut writer = DataWriter::new();
/// writer.start();
///
/// writer.add(DataEntry {
///    // It will be saved to './record/test20190101.csv'
///    // according to our definition in `super::datadir`.
///    filename: "test".to_string(),
///
///    // `.to_vec()` is needed to write it asynchoronously.
///    data: b"OwO".to_vec(),
/// });
/// ```
pub struct DataWriter {
    writer_id: Uuid,

    run_flag: BinaryFlag,
    sender: Sender<DataEntry>,
    receiver: Receiver<DataEntry>,
}

impl DataWriter {
    /// Create a new [`DataWriter`].
    pub fn new() -> DataWriter {
        DataWriter::default()
    }

    /// Push a [`DataEntry`] to write.
    ///
    /// # Example
    ///
    /// ```
    /// use wmjtyd_libstock::file::writer::{DataWriter, DataEntry};
    ///
    /// let mut writer = DataWriter::new();
    ///
    /// writer.add(DataEntry {
    ///    // It will be saved to './record/test20190101.csv'
    ///    // according to our definition in `super::datadir`.
    ///    filename: "test".to_string(),
    ///
    ///    // `.to_vec()` is needed to write it asynchoronously.
    ///    data: b"OwO".to_vec(),
    /// });
    /// ```
    pub fn add(&mut self, data: DataEntry) -> WriteResult<()> {
        tracing::info!("Adding data {data} to writer {writer}", writer = self.writer_id);

        self.sender
            .send(data)
            .map_err(|_| WriteError::PushChannelFailed)
    }

    /// Spawn the writer daemon.
    pub async fn start(&self) -> WriteResult<JoinHandle<()>> {
        let span = tracing::info_span!("DataWriter::start", id = self.writer_id.to_string());
        
        async move {
            let data_dir = get_data_directory();
            tracing::info!("The files will be saved in: {}", data_dir.display());

            if data_dir.exists() {
                tracing::debug!("The data directory has been created. Ignoring.");
            } else {
                tracing::info!("Creating the data directory…");
                tokio::fs::create_dir_all(get_data_directory())
                    .await
                    .map_err(WriteError::DataDirCreationFailed)?;
            }

            let run_flag = self.run_flag.clone();
            let receiver = self.receiver.clone();

            tracing::info!("Starting daemon…");
            Ok(tokio::task::spawn(async move {
                loop {
                    if !run_flag.is_running() {
                        tracing::debug!("Daemon has received stop signal. Exiting.");
                        break;
                    }

                    let task = async {
                        // Get the data entry.
                        let DataEntry { filename, data } = receiver
                            .recv_async()
                            .await
                            .map_err(WriteError::RecvDataFailed)?;

                        tracing::trace!("Received a data entry. Processing…");

                        // Get the timestamp, and get the identifier.
                        let identifier = get_ident(&filename, &get_timestamp());

                        // Write file to the specified path.
                        tracing::debug!("Writing ”{filename}“, data_len: {len}…", len = data.len());
                        let path_to_write = get_ident_path(&identifier);
                        write_content(path_to_write, data.as_slice()).await?;

                        Ok::<(), WriteError>(())
                    };

                    if let Err(e) = task.await {
                        tracing::error!("Error happened: {e}; skipping.");
                        continue;
                    }
                }
            }))
        }
            .instrument(span)
            .await
    }

    /// Stop the writer daemon.
    pub fn stop(&self) {
        tracing::info!("Stopping writer {writer}…", writer = self.writer_id);

        self.run_flag.set_running(false);
    }
}

impl Default for DataWriter {
    fn default() -> Self {
        let (sender, receiver) = flume::unbounded();

        Self {
            // Generate a writer ID for debugging.
            writer_id: Uuid::new_v4(),
            sender,
            receiver,
            run_flag: Default::default(),
        }
    }
}

async fn write_content(path: impl AsRef<std::path::Path>, data: &[u8]) -> WriteResult<()> {
    use tokio::io::AsyncWriteExt;

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .await
        .map_err(WriteError::FileOpenFailed)?;

    // First, write length to file.
    let data_len = data.len().to_be_bytes();
    file.write_all(&data_len)
        .await
        .map_err(WriteError::LengthWriteFailed)?;

    // Then, write data to file.
    file.write_all(data)
        .await
        .map_err(WriteError::DataWriteFailed)?;

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum WriteError {
    #[error("failed to create data directory: {0}")]
    DataDirCreationFailed(tokio::io::Error),

    #[error("failed to push an entry to channel")]
    PushChannelFailed,

    #[error("failed to open file: {0}")]
    FileOpenFailed(tokio::io::Error),

    #[error("failed to write length to file: {0}")]
    LengthWriteFailed(tokio::io::Error),

    #[error("failed to write data to file: {0}")]
    DataWriteFailed(tokio::io::Error),

    #[error("failed to receive data entry: {0}")]
    RecvDataFailed(flume::RecvError),
}

pub type WriteResult<T> = Result<T, WriteError>;
