use tokio::sync::mpsc;
use chrono::Local;
use crate::event::Event;

pub async fn keyboard_task(
    tx: mpsc::Sender<Event>,
) {

    let timestamp = Local::now()
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();

    let _ = tx.send(Event {
        module: "KEYBOARD".into(),
        event_type: "initialized".into(),
        metadata: "linux_stub_loaded".into(),
        timestamp,
    }).await;
}
