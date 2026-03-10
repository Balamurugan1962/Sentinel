use anyhow::Result;
use std::time::Duration;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::{broadcast, mpsc},
    time::sleep,
};

use crate::{config::SharedConfig, user::SharedUser};

pub async fn root_server_task(
    network_tx: mpsc::Sender<String>,
    mut server_rx: mpsc::Receiver<String>,
    user: SharedUser,
    config: SharedConfig,
    mut shutdown: broadcast::Receiver<()>,
) -> Result<()> {
    loop {
        let (ip, port, client_id, version) = {
            let cfg = config.lock().await;
            (
                cfg.server_ip.clone(),
                cfg.server_port.clone(),
                cfg.client_id.clone(),
                cfg.version.clone(),
            )
        };

        let addr = format!("{}:{}", ip, port);

        println!("[SERVER] Trying to connect to {}", addr);

        let mut stream = match TcpStream::connect(&addr).await {
            Ok(s) => {
                println!("[SERVER] Connected to root server");
                s
            }
            Err(e) => {
                println!("[SERVER] Connection failed: {}", e);
                sleep(Duration::from_secs(5)).await;
                continue;
            }
        };

        let hello = format!("HELLO {} {}\n", client_id, version);

        if stream.write_all(hello.as_bytes()).await.is_err() {
            println!("[SERVER] Failed sending HELLO");
            sleep(Duration::from_secs(3)).await;
            continue;
        }

        let mut buffer = [0u8; 1024];

        let n = match stream.read(&mut buffer).await {
            Ok(n) => n,
            Err(e) => {
                println!("[SERVER] Handshake read failed: {}", e);
                sleep(Duration::from_secs(3)).await;
                continue;
            }
        };

        let response = String::from_utf8_lossy(&buffer[..n]);

        if response.trim() != "AKN" {
            println!("[SERVER] Handshake rejected");
            sleep(Duration::from_secs(3)).await;
            continue;
        }

        println!("[SERVER] Handshake success");

        {
            let u = user.lock().await;

            let info = format!(
                "INFO user {}\n{{\"name\":\"{}\",\"regno\":\"{}\"}}\n",
                version, u.name, u.reg
            );

            let _ = stream.write_all(info.as_bytes()).await;
        }

        {
            let u = user.lock().await;

            let info = format!(
                "INFO user {}\n{{\"name\":\"{}\",\"regno\":\"{}\"}}\n",
                version, u.name, u.reg
            );

            println!("Sending user state: {} {}", u.name, u.reg);
            let _ = stream.write_all(info.as_bytes()).await;
        }

        loop {
            tokio::select! {

                _ = shutdown.recv() => {
                    println!("[SERVER] Shutdown signal received");
                    return Ok(());
                }

                result = stream.read(&mut buffer) => {

                    let n = match result {
                        Ok(n) => n,
                        Err(e) => {
                            println!("[SERVER] Read error: {}", e);
                            break;
                        }
                    };

                    if n == 0 {
                        println!("[SERVER] Server disconnected");
                        break;
                    }

                    let message = String::from_utf8_lossy(&buffer[..n]).to_string();

                    println!("[SERVER] {}", message.trim());

                    if message.contains("ACTION self") {
                        println!("[SELF ACTION] {}", message.trim());
                    }

                    else if message.contains("ACTION network") {

                        if let Err(e) = network_tx.send(message.clone()).await {
                            println!("network channel closed {}", e);
                        }
                    }
                }

                Some(msg) = server_rx.recv() => {

                    if let Err(e) = stream.write_all(msg.as_bytes()).await {
                        println!("Write failed {}", e);
                        break;
                    }
                }
            }
        }

        println!("[SERVER] Reconnecting in 5 seconds...");
        sleep(Duration::from_secs(5)).await;
    }
}
