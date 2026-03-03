use anyhow::Result;
use daemonize::Daemonize;
use std::collections::HashMap;
use std::fs::File;
use std::future;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::runtime::Builder;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UnixListener, UnixStream};
use tokio::sync::Mutex;

use tokio::sync::mpsc;

type Clients = Arc<Mutex<HashMap<usize, mpsc::Sender<String>>>>;
static CLIENT_ID: AtomicUsize = AtomicUsize::new(1);

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    daemonize()?;

    let runtime = Builder::new_multi_thread().enable_all().build()?;

    runtime.block_on(async_main())
}

async fn async_main() -> Result<()> {
    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));

    let tcp_clients = clients.clone();
    let unix_clients = clients.clone();

    tokio::spawn(async move {
        if let Err(e) = run_tcp_server(tcp_clients).await {
            eprintln!("TCP server error: {:?}", e);
        }
    });

    tokio::spawn(async move {
        if let Err(e) = run_unix_server(unix_clients).await {
            eprintln!("Unix server error: {:?}", e);
        }
    });

    future::pending::<()>().await;
    Ok(())
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

async fn run_tcp_server(clients: Clients) -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    loop {
        let (stream, addr) = listener.accept().await?;
        let id = CLIENT_ID.fetch_add(1, Ordering::SeqCst);

        println!("Client {} connected from {}", id, addr);

        let (tx, rx) = mpsc::channel::<String>(32);

        clients.lock().await.insert(id, tx);

        tokio::spawn(handle_tcp(id, stream, rx, clients.clone()));
    }
}

async fn handle_tcp(
    id: usize,
    mut stream: TcpStream,
    mut rx: mpsc::Receiver<String>,
    clients: Clients,
) {
    let mut buffer = [0u8; 1024];

    loop {
        tokio::select! {

            // Incoming data from TCP client
            result = stream.read(&mut buffer) => {
                match result {
                    Ok(0) => {
                        println!("Client {} disconnected", id);
                        clients.lock().await.remove(&id);
                        return;
                    }
                    Ok(n) => {
                        println!("From {}: {}", id, String::from_utf8_lossy(&buffer[..n]));
                    }
                    Err(_) => {
                        clients.lock().await.remove(&id);
                        return;
                    }
                }
            }

            // Message from Unix control plane
            Some(msg) = rx.recv() => {
                let _ = stream.write_all(msg.as_bytes()).await;
            }
        }
    }
}

async fn run_unix_server(clients: Clients) -> Result<()> {
    let path = "/tmp/sentinel.sock";

    if std::path::Path::new(path).exists() {
        std::fs::remove_file(path)?;
    }

    let listener = UnixListener::bind(path)?;

    loop {
        let (stream, _) = listener.accept().await?;
        let clients_clone = clients.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_unix(stream, clients_clone).await {
                eprintln!("Unix handler error: {:?}", e);
            }
        });
    }
}

async fn handle_unix(mut stream: UnixStream, clients: Clients) -> Result<()> {
    let mut buffer = [0u8; 1024];
    let n = stream.read(&mut buffer).await?;
    let cmd = String::from_utf8_lossy(&buffer[..n]).trim().to_string();

    match cmd.as_str() {
        "-status" => {
            let response = format!("Status: Active\n");
            stream.write_all(response.as_bytes()).await?;
        }
        "-ls" => {
            let guard = clients.lock().await;
            let list: Vec<String> = guard.keys().map(|id| id.to_string()).collect();
            let response = format!("Connected Clients: {:?}\n", list);
            stream.write_all(response.as_bytes()).await?;
        }

        _ if cmd.starts_with("-send") => {
            let parts: Vec<&str> = cmd.splitn(3, ' ').collect();
            if parts.len() < 3 {
                stream.write_all(b"Usage: -send <id> <message>\n").await?;
                return Ok(());
            }

            let id: usize = parts[1].parse().unwrap_or(0);
            let message = parts[2].to_string();

            let guard = clients.lock().await;

            if let Some(sender) = guard.get(&id) {
                let _ = sender.send(message).await;
                stream.write_all(b"Message sent\n").await?;
            } else {
                stream.write_all(b"Client not found\n").await?;
            }
        }

        _ => {
            stream.write_all(b"Unknown command\n").await?;
        }
    }

    Ok(())
}
