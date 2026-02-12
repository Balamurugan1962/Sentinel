use std::{fs, time::Duration};

use tokio::{
    net::TcpStream,
    time::sleep,
    sync::mpsc,
    io::AsyncWriteExt,
    fs::OpenOptions,
};

use serde::{Deserialize, Serialize};
use chrono::Local;

#[derive(Clone, Deserialize)]
struct Config {
    client_id: String,
    root_ip: String,
    root_port: u16,
    heartbeat_interval: u64,
}

#[derive(Clone, Serialize)]
struct Event {
    module: String,
    event_type: String,
    metadata: String,
    timestamp: String,
}

async fn logger_task(mut rx: mpsc::Receiver<Event>) {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("sentinel.log")
        .await
        .expect("Cannot open log file");

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

async fn heartbeat_task(config: Config, tx: mpsc::Sender<Event>) {

    loop {

        let addr = format!("{}:{}", config.root_ip, config.root_port);

        let timestamp = Local::now()
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();

        let heartbeat_payload = serde_json::json!({
            "type": "heartbeat",
            "client_id": config.client_id,
            "timestamp": timestamp
        });

        match TcpStream::connect(&addr).await {

            Ok(mut stream) => {

                let message = format!("{}\n", heartbeat_payload.to_string());

                if let Err(_) = stream.write_all(message.as_bytes()).await {

                    let _ = tx.send(Event {
                        module: "NETWORK".into(),
                        event_type: "send_failed".into(),
                        metadata: addr.clone(),
                        timestamp,
                    }).await;

                } else {

                    let _ = tx.send(Event {
                        module: "NETWORK".into(),
                        event_type: "heartbeat_sent".into(),
                        metadata: addr.clone(),
                        timestamp,
                    }).await;
                }
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


async fn network_monitor_task(tx: mpsc::Sender<Event>) {

    let mut last_status: Option<bool> = None;

    loop {

        let status = TcpStream::connect("1.1.1.1:53")
            .await
            .is_ok();

        if last_status != Some(status) {

            last_status = Some(status);

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

        sleep(Duration::from_secs(3)).await;
    }
}

async fn keyboard_task(tx: mpsc::Sender<Event>) {

    let timestamp = Local::now()
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();

    let _ = tx.send(Event {
        module: "KEYBOARD".into(),
        event_type: "initialized".into(),
        metadata: "module_loaded".into(),
        timestamp,
    }).await;
}

async fn mouse_task(tx: mpsc::Sender<Event>) {

    let timestamp = Local::now()
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();

    let _ = tx.send(Event {
        module: "MOUSE".into(),
        event_type: "initialized".into(),
        metadata: "module_loaded".into(),
        timestamp,
    }).await;
}

#[tokio::main]
async fn main() {

    let config_data =
        fs::read_to_string("config.toml")
            .expect("Cannot read config.toml");

    let config: Config =
        toml::from_str(&config_data)
            .expect("Invalid TOML format");

    println!("Sentinel Rust Client Started");
    println!("Client ID: {}", config.client_id);

    let (tx, rx) = mpsc::channel(100);

    tokio::spawn(logger_task(rx));

    let _ = tx.send(Event {
        module: "SYSTEM".into(),
        event_type: "startup".into(),
        metadata: "client_started".into(),
        timestamp: Local::now()
            .format("%Y-%m-%d %H:%M:%S")
            .to_string(),
    }).await;

    tokio::spawn(heartbeat_task(config.clone(), tx.clone()));
    tokio::spawn(network_monitor_task(tx.clone()));
    tokio::spawn(keyboard_task(tx.clone()));
    tokio::spawn(mouse_task(tx.clone()));

    loop {
        sleep(Duration::from_secs(3600)).await;
    }
}
