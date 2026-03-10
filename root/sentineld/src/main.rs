use anyhow::Result;
use daemonize::Daemonize;
use std::collections::HashMap;
use std::fs::File;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime::Builder;
use tokio::sync::{broadcast, mpsc, Mutex};

use crate::bridge::main::start_http;

struct ClientMeta {
    tx: mpsc::Sender<String>,
    name: String,
    reg: String,
}

mod bridge;

type Clients = Arc<Mutex<HashMap<usize, ClientMeta>>>;
static CLIENT_COUNTER: AtomicUsize = AtomicUsize::new(1);

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    daemonize()?;

    let runtime = Builder::new_multi_thread().enable_all().build()?;
    runtime.block_on(async_main())
}

fn daemonize() -> Result<()> {
    let stdout = File::create("/tmp/sentinel.out")?;
    let stderr = File::create("/tmp/sentinel.err")?;

    let daemonize = Daemonize::new()
        .pid_file("/tmp/sentinel.pid")
        .stdout(stdout)
        .stderr(stderr);

    daemonize.start()?;
    Ok(())
}

async fn async_main() -> Result<()> {
    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));
    let (shutdown_tx, _) = broadcast::channel::<()>(1);

    let tcp_clients = clients.clone();
    let unix_clients = clients.clone();

    let tcp_shutdown = shutdown_tx.subscribe();
    let unix_shutdown = shutdown_tx.subscribe();

    tokio::spawn(async move {
        if let Err(e) = run_tcp_server(tcp_clients, tcp_shutdown).await {
            eprintln!("TCP server error: {:?}", e);
        }
    });

    let shutdown_tx_unix = shutdown_tx.clone();

    tokio::spawn(async move {
        if let Err(e) = start_http(unix_clients, shutdown_tx_unix, unix_shutdown).await {
            eprintln!("Unix server error: {:?}", e);
        }
    });

    shutdown_tx.subscribe().recv().await.ok();
    println!("Sentinel shutting down...");
    Ok(())
}

async fn run_tcp_server(clients: Clients, mut shutdown: broadcast::Receiver<()>) -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:1612").await?;

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
