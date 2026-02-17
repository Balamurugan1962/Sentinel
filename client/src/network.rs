use tokio::net::TcpStream;
use tokio::time::{sleep, Duration};
use tokio::sync::mpsc;
use chrono::Local;
use crate::event::Event;

pub async fn network_monitor_task(
    tx: mpsc::Sender<Event>,
) {

    let mut network_up: Option<bool> = None;

    loop {

        let status = TcpStream::connect("8.8.8.8:53")
            .await
            .is_ok();

        if network_up != Some(status) {

            network_up = Some(status);

            let timestamp = Local::now()
                .format("%Y-%m-%d %H:%M:%S")
                .to_string();

            let _ = tx.send(Event {
                module: "NETWORK".into(),
                event_type: "network_status".into(),
                metadata: if status { "up".into() } else { "down".into() },
                timestamp,
            }).await;
        }

        sleep(Duration::from_secs(5)).await;
    }
}
