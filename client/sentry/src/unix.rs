use anyhow::Result;
use std::path::Path;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{UnixListener, UnixStream},
    sync::{broadcast, mpsc},
};

use crate::{config::SharedConfig, user::SharedUser};

pub async fn run_unix_server(
    shutdown_tx: broadcast::Sender<()>,
    mut shutdown: broadcast::Receiver<()>,
    user: SharedUser,
    config: SharedConfig,
    server_tx: mpsc::Sender<String>,
) -> Result<()> {
    let unix_socket = config.lock().await.unix_socket.clone();

    if Path::new(&unix_socket).exists() {
        std::fs::remove_file(&unix_socket)?;
    }

    let listener = UnixListener::bind(&unix_socket)?;

    println!("Unix control socket ready {}", unix_socket);

    loop {
        tokio::select! {

            _ = shutdown.recv() => {
                println!("Unix server shutting down");
                break;
            }

            result = listener.accept() => {

                let (stream, _) = result?;

                let shutdown_clone = shutdown_tx.clone();
                let user_clone = user.clone();
                let server_tx_clone = server_tx.clone();
                let config_clone = config.clone();

                tokio::spawn(async move {

                    if let Err(e) = handle_unix(
                        stream,
                        shutdown_clone,
                        user_clone,
                        config_clone,
                        server_tx_clone
                    ).await {

                        eprintln!("Unix handler error {:?}", e);
                    }

                });
            }
        }
    }

    Ok(())
}

async fn handle_unix(
    mut stream: UnixStream,
    shutdown_tx: broadcast::Sender<()>,
    user: SharedUser,
    config: SharedConfig,
    server_tx: mpsc::Sender<String>,
) -> Result<()> {
    let mut buffer = [0u8; 1024];

    let n = stream.read(&mut buffer).await?;
    let cmd = String::from_utf8_lossy(&buffer[..n]).trim().to_string();

    if cmd.starts_with("info") {
        let mut name: Option<String> = Some("unknown".to_string());
        let mut reg: Option<String> = Some("unknown".to_string());

        let parts: Vec<&str> = cmd.split_whitespace().collect();

        let mut i = 1;
        while i < parts.len() {
            match parts[i] {
                "--name" if i + 1 < parts.len() => {
                    name = Some(parts[i + 1].to_string());
                    i += 2;
                }
                "--reg" if i + 1 < parts.len() => {
                    reg = Some(parts[i + 1].to_string());
                    i += 2;
                }
                _ => {
                    i += 1;
                }
            }
        }

        {
            let mut u = user.lock().await;

            if let Some(n) = name {
                u.name = n;
            }

            if let Some(r) = reg {
                u.reg = r;
            }

            let info = format!(
                "INFO user {}\n{{\"name\":\"{}\",\"regno\":\"{}\"}}\n",
                config.lock().await.version,
                u.name,
                u.reg
            );

            server_tx.send(info).await?;
        }

        stream.write_all(b"Info updated\n").await?;
    } else if cmd == "-logout" {
        let name: Option<String> = Some("unknown".to_string());
        let reg: Option<String> = Some("unknown".to_string());

        {
            let mut u = user.lock().await;

            if let Some(n) = name {
                u.name = n;
            }

            if let Some(r) = reg {
                u.reg = r;
            }

            let info = format!(
                "INFO user {}\n{{\"name\":\"{}\",\"regno\":\"{}\"}}\n",
                config.lock().await.version,
                u.name,
                u.reg
            );

            server_tx.send(info).await?;
        }

        stream.write_all(b"Info updated\n").await?;
    } else if cmd == "-status" {
        stream.write_all(b"Sentry running\n").await?;
    } else if cmd == "-stop" {
        stream.write_all(b"Stopping sentry\n").await?;
        let _ = shutdown_tx.send(());
    } else {
        stream.write_all(b"Unknown command\n").await?;
    }

    Ok(())
}
