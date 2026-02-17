use tokio::sync::mpsc;
use tokio::io::AsyncWriteExt;
use tokio::fs::OpenOptions;
use crate::event::Event;

pub async fn logger_task(mut rx: mpsc::Receiver<Event>) {

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("sentinel.log")
        .await
        .expect("Failed to open log file");

    while let Some(event) = rx.recv().await {

        let line = format!(
            "{}|{}|{}|{}\n",
            event.module,
            event.timestamp,
            event.event_type,
            event.metadata
        );

        let _ = file.write_all(line.as_bytes()).await;
    }
}
