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
    println!("[SERVER] Connecting to root server...");

    let (ip, port, client_id, version) = {
        let cfg = config.lock().await;
        (
            cfg.server_ip.clone(),
            cfg.server_port.clone(),
            cfg.client_id.clone(),
            cfg.version.clone(),
        )
    };

    let mut stream = TcpStream::connect(format!("{}:{}", ip, port)).await?;

    let hello = format!("HELLO {} {}\n", client_id, version);

    stream.write_all(hello.as_bytes()).await?;

    let mut buffer = [0u8; 1024];

    let n = stream.read(&mut buffer).await?;
    let response = String::from_utf8_lossy(&buffer[..n]);

    if response.trim() != "AKN" {
        println!("Handshake failed");
        return Ok(());
    }

    println!("[SERVER] Handshake success");

    let u = user.lock().await;

    let info = format!(
        "INFO user {}\n{{\"name\":\"{}\",\"regno\":\"{}\"}}\n",
        version, u.name, u.reg
    );

    stream.write_all(info.as_bytes()).await?;

    drop(u);

    loop {
        tokio::select! {

            _ = shutdown.recv() => {
                println!("Server task shutting down");
                break;
            }

            result = stream.read(&mut buffer) => {

                let n = result?;

                if n == 0 {
                    println!("Server disconnected");
                    break;
                }

                let message = String::from_utf8_lossy(&buffer[..n]).to_string();

                println!("[SERVER] {}", message.trim());

                if message.contains("ACTION self") {
                    println!("[SELF ACTION] {}", message.trim());
                }

                else if message.contains("ACTION network") {
                    network_tx.send(message.clone()).await?;
                }
            }

            Some(msg) = server_rx.recv() => {
                stream.write_all(msg.as_bytes()).await?;
            }
        }

        sleep(Duration::from_secs(2)).await;
    }

    Ok(())
}
