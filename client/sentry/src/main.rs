use anyhow::Result;
use std::time::Duration;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::mpsc,
    time::sleep,
};

//TODO:
// should get it from config
const CLIENT_ID: &str = "3122235001027";
const VERSION: &str = "SNT0.1";

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<()> {
    println!("Sentinel Client Starting...");

    // Channel between server thread and network logger
    let (tx, rx) = mpsc::channel::<String>(100);

    // Spawn root server polling thread
    let server_task = tokio::spawn(root_server_task(tx));

    // Spawn network logger thread
    let network_task = tokio::spawn(network_logger_task(rx));

    tokio::try_join!(server_task, network_task)?;

    Ok(())
}

async fn root_server_task(tx: mpsc::Sender<String>) -> Result<()> {
    println!("[SERVER THREAD] Connecting to root server...");

    let mut stream = TcpStream::connect("127.0.0.1:1612").await?;

    // ---- Handshake ----
    let hello = format!("HELLO {} {}\n", CLIENT_ID, VERSION);
    stream.write_all(hello.as_bytes()).await?;

    let mut buffer = [0u8; 1024];
    let n = stream.read(&mut buffer).await?;
    let response = String::from_utf8_lossy(&buffer[..n]);

    if response.trim() != "AKN" {
        println!("Handshake failed");
        return Ok(());
    }

    println!("[SERVER THREAD] Handshake successful");

    // ---- Polling Loop ----
    loop {
        let n = stream.read(&mut buffer).await?;

        if n == 0 {
            println!("Server disconnected");
            break;
        }

        let message = String::from_utf8_lossy(&buffer[..n]).to_string();

        if message.contains("ACTION self") {
            println!("[SERVER THREAD] {}", message.trim());
        } else if message.contains("ACTION network") {
            tx.send(message.clone()).await?;
        }

        sleep(Duration::from_secs(2)).await;
    }

    Ok(())
}

async fn network_logger_task(mut rx: mpsc::Receiver<String>) -> Result<()> {
    println!("[NETWORK THREAD] Logger started");

    // Mock Kafka Producer
    // In real world use rdkafka FutureProducer

    while let Some(message) = rx.recv().await {
        println!("[NETWORK THREAD] {}", message.trim());

        // Mock sending to Kafka
        println!("[NETWORK THREAD] Sending mock data to Kafka...");
        sleep(Duration::from_millis(500)).await;
        println!("[NETWORK THREAD] Kafka send complete");
    }

    Ok(())
}
