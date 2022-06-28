//! The writer daemon to write data and place file automatically
//! without worrying about managing the path.

use std::{sync::{atomic::AtomicBool, Arc}, path::PathBuf};

use concat_string::concat_string;
use flume::{Sender, Receiver};
use tokio::{task::JoinHandle, fs::OpenOptions};

/// A data entry to send to a [`DataWriter`].
pub struct DataEntry {
    /// The file name to write as.
    pub filename: String,
    /// The data to write.
    pub data: Vec<u8>,
}

#[derive(Clone)]
pub struct RunningFlag(Arc<AtomicBool>);

pub struct DataWriter {
    run_flag: RunningFlag,
    sender: Sender<DataEntry>,
    receiver: Receiver<DataEntry>,
}

impl RunningFlag {
    pub fn new() -> Self {
        Self::default()
    }

    /// Should we continue running?
    pub fn is_running(&self) -> bool {
        self.0.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Set the running flag.
    pub fn set_running(&self, value: bool) {
        self.0.store(value, std::sync::atomic::Ordering::SeqCst)
    }
}

impl DataWriter {
    /// Create a new [`DataWriter`].
    pub fn new() -> DataWriter {
        DataWriter::default()
    }

    /// Push a [`DataEntry`] to write.
    pub fn add(&mut self, data: DataEntry) -> WriteResult<()> {
        self.sender
            .send(data)
            .map_err(|_| WriteError::PushChannelFailed)
    }

    /// Spawn the writer daemon.
    pub async fn start(&mut self) -> WriteResult<JoinHandle<()>> {
        // Create the directory to place the files in.
        tokio::fs::create_dir_all(get_data_directory())
            .await
            .map_err(WriteError::DataDirCreationFailed)?;

        let run_flag = self.run_flag.clone();
        let receiver = self.receiver.clone();
        
        Ok(tokio::task::spawn(async move {
            loop {
                if !run_flag.is_running() {
                    break;
                }
                
                let task = async {
                    // Get the data entry.
                    let DataEntry { filename, data } = 
                        receiver
                            .recv_async()
                            .await
                            .map_err(WriteError::RecvDataFailed)?;
                    
                    // Get the timestamp, and get the identifier.
                    let timestamp = get_timestamp();
                    let identifier = concat_string!(filename, timestamp);

                    // Write file to the specified path.
                    let path_to_write = get_path_to_write(&identifier);
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
}

impl Default for RunningFlag {
    fn default() -> Self {
        Self(Arc::new(AtomicBool::new(true)))
    }
}

impl Default for DataWriter {
    fn default() -> Self {
        let (sender, receiver) = flume::unbounded();

        Self {
            sender,
            receiver,
            run_flag: RunningFlag::default(),
        }
    }
}

/// Get a timestamp whose format is `%Y%m%d`.
fn get_timestamp() -> String {
    use chrono::Local;

    let local_time = Local::now();
    local_time.format("%Y%m%d").to_string()
}

/// Get the data directory.
/// 
/// Currently, the data directory is `./record`.
fn get_data_directory() -> PathBuf {
    let mut path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    path.push("record");
    path
}

/// Get the exact filename to write to.
fn get_path_to_write(identifier: &str) -> PathBuf {
    let mut path = get_data_directory();
    path.push(concat_string!(identifier, ".csv"));

    path
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
