use std::{sync::atomic::AtomicBool, path::{PathBuf}};

use concat_string::concat_string;
use flume::{Sender, Receiver};
use scc::HashMap;
use tokio::{task::JoinHandle, fs::{File, OpenOptions}};

/// A data entry to send to a [`DataWriter`].
pub struct DataEntry {
    /// The file name to write as.
    pub filename: String,
    /// The data to write.
    pub data: Vec<u8>,
}

pub struct RunningFlag(AtomicBool);

pub struct DataWriter {
    run_flag: RunningFlag,
    sender: Sender<DataEntry>,
    receiver: Receiver<DataEntry>,
    files: HashMap<String, File>,
}

impl RunningFlag {
    pub fn new() -> Self {
        Self::default()
    }

    /// Should we continue running?
    pub fn is_running(&self) -> bool {
        self.0.load(std::sync::atomic::Ordering::Release)
    }

    /// Set the running flag.
    pub fn set_running(&self, value: bool) {
        self.0.store(value, std::sync::atomic::Ordering::Acquire)
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
            .map_err(|e| WriteError::PushChannelFailed)
    }

    /// Spawn the writer daemon.
    pub async fn start(&mut self) -> JoinHandle<()> {
        // Create the directory to place the files in.
        tokio::fs::create_dir_all(get_data_directory()).await;

        tokio::task::spawn(async {
            loop {
            if !self.run_flag.is_running() {
                break;
            }

            let timestamp = get_timestamp();
            let DataEntry { filename, data } = 
                match self.receiver.recv_async().await {
                    Ok(data_entry) => data_entry,
                    Err(e) => {
                        tracing::error!("error receiving data entry: {e}. skipping.");
                        continue;
                    }
                };

            let identifier = concat_string!(filename, timestamp);
            let file_info = self.files.read_async(&identifier, |_, &v| v).await;
    
            match file_info {
                // No such a file!
                None => {
                    let path_to_write = get_path_to_write(&identifier);
                    let result = write_content(path_to_write, data.as_slice()).await;

                    if let Err(e) = result {
                        tracing::error!("{e}. skipping.");
                        continue;
                    }
                }
            }

            match files_lock.get(&format!("{}_{}", filename, local_time)) {
                None => {

                    let filename_orderbook = check_path(filename.to_string());
                    let data_file = OpenOptions::new()
                        .append(true)
                        .open(filename_orderbook)
                        .expect("文件无法打开");

                    write_file(&data_file, data);
                    files_lock.insert(today, data_file);

                    let yesterday = (local_time - Duration::seconds(86400)).format("%Y%m%d");
                    let key = &format!("{}_{}", filename, yesterday);
                    if files_lock.contains_key(key) {
                        files_lock.remove(key);
                    }
                }
                Some(file) => {
                    write_file(file, data);
                }
            }
        }
        })
    }

    async fn loop_task(&mut self) -> WriteResult<()> {
        let timestamp = get_timestamp();
        let DataEntry { filename, data } = 
            self.receiver
                .recv_async()
                .await
                .map_err(WriteError::RecvDataFailed)?;

        let identifier = concat_string!(filename, timestamp);
        let file_info = self.files.read_async(
            &identifier,
            |_, &v| v.clone()
        ).await;

        match file_info {
            // No such a file!
            None => {
                let path_to_write = get_path_to_write(&identifier);
                let result = write_content(path_to_write, data.as_slice()).await?;
            }
        }

        todo!()
    }
}

impl Default for RunningFlag {
    fn default() -> Self {
        Self(AtomicBool::new(true))
    }
}

impl Default for DataWriter {
    fn default() -> Self {
        let (sender, receiver) = flume::unbounded();

        Self {
            sender,
            receiver,
            ..Default::default()
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
    let path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    path.push("record");
    path
}

/// Get the exact filename to write to.
fn get_path_to_write(identifier: &str) -> PathBuf {
    let path = get_data_directory();
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
        .map_err(|e| WriteError::FileOpenFailed(e))?;
    
    // First, write length to file.
    let data_len = data.len().to_be_bytes();
    file.write_all(&data_len)
        .await
        .map_err(|e| WriteError::LengthWriteFailed(e))?;

    // Then, write data to file.
    file.write_all(&data)
        .await
        .map_err(|e| WriteError::DataWriteFailed(e))?;

    Ok(())
}

// 可优化
pub fn write_file(mut file: &File, data: Vec<i8>) {
    let data_len = &(data.len() as i16).to_be_bytes();
    file.write_all(data_len).expect("长度计算失败");
    for i in data {
        file.write_all(&[i as u8]).expect("");
    }
}

pub fn check_path(filename: String) -> String {
    let local_time = Local::now().format("%Y%m%d").to_string();
    let path = &format!("./record/{}/", local_time);
    create_dir_all(path).expect("目录创建失败");

    let path_filename = format!("{}{}.csv", path, filename);
    println!("{}", path_filename);
    if !Path::new(&path_filename).is_file() {
        File::create(&path_filename).expect("创建文件失败");
    }
    path_filename
}

#[derive(Debug, thiserror::Error)]
pub enum WriteError {
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
