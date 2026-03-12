use anyhow::Result;
use std::sync::atomic::Ordering;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc};

use crate::config::SharedConfig;
use crate::{ClientMeta, Clients, CLIENT_COUNTER};

pub async fn run_tcp_server(
    config: SharedConfig,
    clients: Clients,
    mut shutdown: broadcast::Receiver<()>,
) -> Result<()> {
    let server_ip = config.lock().await.server_ip.clone();
    let addr = format!("{}:1612", server_ip);
    let listener = TcpListener::bind(addr).await?;

    loop {
        tokio::select! {

            _ = shutdown.recv() => {
                println!("TCP server shutting down...");
                break;
            }

            result = listener.accept() => {
                let (mut stream, _) = result?;

                let mut buffer = [0u8;1024];
                let n = stream.read(&mut buffer).await?;
                let msg = String::from_utf8_lossy(&buffer[..n]).to_string();

                let parts:Vec<&str> = msg.trim().split_whitespace().collect();

                let mut id = CLIENT_COUNTER.fetch_add(1, Ordering::SeqCst);

                if parts.len() == 3 && parts[0] == "HELLO" {
                    id = parts[1].parse().unwrap_or(id);
                    stream.write_all(b"AKN").await?;
                }

                let (tx,rx) = mpsc::channel::<String>(32);

                clients.lock().await.insert(
                    id,
                    ClientMeta{
                        tx,
                        name:"unknown".into(),
                        reg:"unknown".into()
                    }
                );

                println!("Client {} connected",id);

                tokio::spawn(handle_tcp(id,stream,rx,clients.clone()));
            }
        }
    }

    Ok(())
}

async fn handle_tcp(
    id: usize,
    stream: TcpStream,
    mut rx: mpsc::Receiver<String>,
    clients: Clients,
) {
    let (read_half, mut write_half) = stream.into_split();
    let mut reader = BufReader::new(read_half);
    let mut line = String::new();

    loop {
        tokio::select! {

            result = reader.read_line(&mut line) => {

                match result {

                    Ok(0) => {
                        println!("Client {} disconnected", id);
                        clients.lock().await.remove(&id);
                        return;
                    }

                    Ok(_) => {

                        let message = line.trim().to_string();
                        line.clear();

                        println!("From {}: {}", id, message);

                        if message.starts_with('{') {

                            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&message) {

                                let mut guard = clients.lock().await;

                                if let Some(meta) = guard.get_mut(&id) {

                                    if let Some(name) = v["name"].as_str() {
                                        meta.name = name.to_string();
                                    }

                                    if let Some(reg) = v["regno"].as_str() {
                                        meta.reg = reg.to_string();
                                    }

                                    println!("Updated client {} -> {} {}", id, meta.name, meta.reg);
                                }
                            }
                        }
                    }

                    Err(_) => {
                        clients.lock().await.remove(&id);
                        return;
                    }
                }
            }

            Some(msg) = rx.recv() => {
                let _ = write_half.write_all(msg.as_bytes()).await;
            }
        }
    }
}
