use tokio::net::TcpStream;
use tokio::time::{sleep, Duration};
use tokio::sync::mpsc;
use tokio::io::AsyncWriteExt;
use chrono::Local;
use crate::{config::Config, event::Event};

pub async fn heartbeat_task(
    config: Config,
    tx: mpsc::Sender<Event>,
) {

    loop {

        let addr = format!("{}:{}", config.root_ip, config.root_port);

        let timestamp = Local::now()
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();

        match TcpStream::connect(&addr).await {

            Ok(mut stream) => {

                let _ = stream
                    .write_all(config.client_id.as_bytes())
                    .await;

                let _ = tx.send(Event {
                    module: "NETWORK".into(),
                    event_type: "heartbeat_sent".into(),
                    metadata: config.client_id.clone(),
                    timestamp,
                }).await;
            }

            Err(_) => {

                let _ = tx.send(Event {
                    module: "NETWORK".into(),
                    event_type: "root_unreachable".into(),
                    metadata: addr.clone(),
                    timestamp,
                }).await;
            }
        }

        sleep(Duration::from_secs(config.heartbeat_interval)).await;
    }
}
