//! The writer daemon to write data and place file automatically
//! without worrying about managing the path.

use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

use flume::{Receiver, Sender};
use tokio::{fs::OpenOptions, task::JoinHandle};
use tracing::Instrument;
use uuid::Uuid;

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

/// The action to pass to the writer daemon channel.
#[non_exhaustive]
enum WriterAction {
    /// Stop daemon.
    Stop,

    /// Send [`DataEntry`] to the daemon to write.
    FileWrite(DataEntry),
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

    sender: Sender<WriterAction>,
    receiver: Receiver<WriterAction>,
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
        tracing::info!(
            "Adding data {data} to writer {writer}",
            writer = self.writer_id
        );

        self.sender
            .send(WriterAction::FileWrite(data))
            .map_err(|_| WriteError::PushChannelFailed)
    }

    /// Spawn the writer daemon.
    pub async fn start(&self) -> WriteResult<JoinHandle<()>> {
        let span = tracing::info_span!(
            "DataWriter::start",
            id = self.writer_id.to_string().as_str()
        );

        async move {
            let data_dir = Self::get_data_dir();
            Self::create_data_dir(data_dir.as_path()).await?;

            let receiver = self.receiver.clone();

            tracing::info!("Starting daemon…");
            let span = tracing::info_span!("daemon");
            Ok(tokio::task::spawn(
                async move {
                    loop {
                        let task = async {
                            let action = receiver
                                .recv_async()
                                .await
                                .map_err(DaemonError::RecvActionFailed)?;

                            Self::process_action(action).await
                        };

                        if let Err(e) = task.await {
                            match e {
                                DaemonError::StopDaemon => {
                                    tracing::trace!("Received the forwarded “StopDaemon” request.");
                                    break;
                                }
                                _ => {
                                    tracing::error!("Error happened: {e}; skipping.");
                                    continue;
                                }
                            }
                        }
                    }
                }
                .instrument(span),
            ))
        }
        .instrument(span)
        .await
    }

    /// Stop the writer daemon.
    pub fn stop(&self) -> WriteResult<()> {
        tracing::info!("Stopping writer {writer}…", writer = self.writer_id);

        self.sender
            .send(WriterAction::Stop)
            .map_err(|_| WriteError::PushChannelFailed)
    }

    fn get_data_dir() -> PathBuf {
        let data_dir = get_data_directory();
        tracing::info!("The files will be saved in: {}", data_dir.display());

        data_dir
    }

    async fn create_data_dir(data_dir: &Path) -> WriteResult<()> {
        if data_dir.exists() {
            tracing::debug!("The data directory has been created. Ignoring.");
        } else {
            tracing::info!("Creating the data directory…");
            tokio::fs::create_dir_all(data_dir)
                .await
                .map_err(WriteError::DataDirCreationFailed)?;
        }

        Ok(())
    }

    async fn process_action(action: WriterAction) -> Result<(), DaemonError> {
        match action {
            WriterAction::FileWrite(DataEntry { filename, data }) => {
                tracing::trace!("Received a data entry. Processing…");

                // Get the timestamp, and get the identifier.
                let identifier = get_ident(&filename, &get_timestamp());

                // Write file to the specified path.
                tracing::debug!("Writing ”{filename}“, data_len: {len}…", len = data.len());
                let path_to_write = get_ident_path(&identifier);
                write_content(path_to_write, data.as_slice()).await?;
            }
            WriterAction::Stop => {
                tracing::debug!("Daemon has received stop signal. Exiting.");
                return Err(DaemonError::StopDaemon);
            }
        }

        Ok(())
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
    let data_len = (data.len() as u16).to_be_bytes();
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
}

#[derive(Debug, thiserror::Error)]
enum DaemonError {
    #[error("writer error: {0}")]
    WriterError(#[from] WriteError),

    #[error("failed to receive action: {0}")]
    RecvActionFailed(flume::RecvError),

    #[error("received stop signal")]
    StopDaemon,
}

pub type WriteResult<T> = Result<T, WriteError>;
