use anyhow::Result;
use daemonize::Daemonize;
use std::collections::HashMap;
use std::fs::File;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UnixListener, UnixStream};
use tokio::runtime::Builder;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::sync::Mutex;

// TODO:
// instead fo string, we need to
// create a custom protocol type

struct ClientMeta {
    tx: mpsc::Sender<String>,
    name: String,
    reg: String,
}

type Clients = Arc<Mutex<HashMap<usize, ClientMeta>>>;
static CLIENT_ID: AtomicUsize = AtomicUsize::new(1);

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    daemonize()?;

    let runtime = Builder::new_multi_thread().enable_all().build()?;

    runtime.block_on(async_main())
}

// TODO:
// for now it logs in /tmp
// but for prod we need to switch it to /var/log/sentinel
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
        if let Err(e) = run_unix_server(unix_clients, shutdown_tx_unix, unix_shutdown).await {
            eprintln!("Unix server error: {:?}", e);
        }
    });

    // Wait until shutdown signal is received
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
                        break Ok(());
            }

            result = listener.accept() => {
                        let (mut stream, addr) = result?;

                        let mut buffer = [0u8; 1024];
                        let n = stream.read(&mut buffer).await?;
                        let message = String::from_utf8_lossy(&buffer[..n]).to_string();

                        let parts: Vec<&str> = message.trim().split_whitespace().collect();
                        let mut id = CLIENT_ID.fetch_add(1, Ordering::SeqCst);

                        if parts.len() == 3 && parts[0] == "HELLO" {
                            id = parts[1].parse::<usize>().unwrap();
                            stream.write_all(b"AKN").await?;
                        }

                        let (tx, rx) = mpsc::channel::<String>(32);
                        clients.lock().await.insert(
                            id,
                            ClientMeta {
                                tx,
                                name: "unknown".into(),
                                reg: "unknown".into(),
                            },
                        );

                        tokio::spawn(handle_tcp(id, stream, rx, clients.clone()));
            }
        }
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

async fn run_unix_server(
    clients: Clients,
    shutdown_tx: broadcast::Sender<()>,
    mut shutdown: broadcast::Receiver<()>,
) -> Result<()> {
    let path = "/tmp/sentinel.sock";

    if std::path::Path::new(path).exists() {
        std::fs::remove_file(path)?;
    }

    let listener = UnixListener::bind(path)?;

    loop {
        tokio::select! {
            _ = shutdown.recv() => {
                println!("Unix server shutting down...");
                break Ok(());
            }

            result = listener.accept() => {
                let (stream, _) = result?;
                let clients_clone = clients.clone();
                let shutdown_clone = shutdown_tx.clone();

                tokio::spawn(async move {
                    if let Err(e) = handle_unix(stream, clients_clone, shutdown_clone).await {
                        eprintln!("Unix handler error: {:?}", e);
                    }
                });
            }
        }
    }
}

async fn handle_unix(
    mut stream: UnixStream,
    clients: Clients,
    shutdown_tx: broadcast::Sender<()>,
) -> Result<()> {
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

            let list: Vec<_> = guard
                .iter()
                .map(|(id, meta)| {
                    serde_json::json!({
                        "id": id,
                        "name": meta.name,
                        "reg": meta.reg
                    })
                })
                .collect();

            let json = serde_json::to_string(&list)?;
            stream.write_all(json.as_bytes()).await?;
        }

        "-stop" => {
            stream.write_all(b"Stopping Sentinel...\n").await?;
            let _ = shutdown_tx.send(());
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

            // if let Some(sender) = guard.get(&id) {
            //     let _ = sender.send(message).await;
            //     stream.write_all(b"Message sent\n").await?;
            // } else {
            //     stream.write_all(b"Client not found\n").await?;
            // }
        }

        _ => {
            stream.write_all(b"Unknown command\n").await?;
        }
    }

    Ok(())
}
