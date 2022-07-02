use std::time::Duration;

use wmjtyd_libstock::file::writer::{DataEntry, DataWriter};

#[tokio::main]
async fn main() {
    let mut writer = DataWriter::new();
    tracing_subscriber::fmt::init();

    let work = writer.start().await.expect("failed to start writer");

    writer
        .add(DataEntry {
            // It will be saved to './record/test20190101.csv'
            // according to our definition in `super::datadir`.
            filename: "test".to_string(),

            // `.to_vec()` is needed to write it asynchoronously.
            data: b"OwO".to_vec(),
        })
        .expect("failed to create writer");

    tracing::info!("Wait 5 seconds…");
    tokio::time::sleep(Duration::from_secs(5)).await;

    writer.stop().expect("failed to send stop request");
    work.await.expect("failed to stop writer");
}
