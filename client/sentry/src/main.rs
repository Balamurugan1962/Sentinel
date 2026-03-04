use anyhow::Result;
use daemonize::Daemonize;
use std::{fs::File, net::Ipv4Addr, path::Path, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpStream, UnixListener, UnixStream},
    runtime::Builder,
    sync::{broadcast, mpsc},
    time::sleep,
};

const CLIENT_ID: &str = "27";
const VERSION: &str = "SNT0.1";
const UNIX_SOCKET: &str = "/tmp/sentry.sock";

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    daemonize()?;

    let runtime = Builder::new_multi_thread().enable_all().build()?;
    runtime.block_on(async_main())
}

fn daemonize() -> Result<()> {
    let stdout = File::create("/tmp/sentry.out")?;
    let stderr = File::create("/tmp/sentry.err")?;

    let daemon = Daemonize::new()
        .pid_file("/tmp/sentry.pid")
        .stdout(stdout)
        .stderr(stderr);

    daemon.start()?;
    Ok(())
}

async fn async_main() -> Result<()> {
    println!("Sentry daemon starting");

    let (tx, rx) = mpsc::channel::<String>(100);
    let (shutdown_tx, _) = broadcast::channel::<()>(1);

    let shutdown_root = shutdown_tx.subscribe();
    let shutdown_unix = shutdown_tx.subscribe();

    tokio::spawn(root_server_task(tx, shutdown_root));
    tokio::spawn(network_logger_task(rx));

    run_unix_server(shutdown_tx, shutdown_unix).await?;

    Ok(())
}

async fn root_server_task(
    tx: mpsc::Sender<String>,
    mut shutdown: broadcast::Receiver<()>,
) -> Result<()> {
    println!("[SERVER] Connecting to root server...");

    let mut stream = TcpStream::connect("127.0.0.1:1612").await?;

    let hello = format!("HELLO {} {}\n", CLIENT_ID, VERSION);
    stream.write_all(hello.as_bytes()).await?;

    let mut buffer = [0u8; 1024];
    let n = stream.read(&mut buffer).await?;
    let response = String::from_utf8_lossy(&buffer[..n]);

    if response.trim() != "AKN" {
        println!("Handshake failed");
        return Ok(());
    }

    println!("[SERVER] Handshake success");

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

                if message.contains("ACTION self") {
                    println!("{}", message.trim());
                }

                else if message.contains("ACTION network") {
                    tx.send(message.clone()).await?;
                }
            }
        }

        sleep(Duration::from_secs(2)).await;
    }

    Ok(())
}

async fn network_logger_task(mut rx: mpsc::Receiver<String>) -> Result<()> {
    println!("[NETWORK] Firewall ready");

    while let Some(message) = rx.recv().await {
        println!("[NETWORK] {}", message.trim());

        if let Some(ip_str) = parse_block_ip(&message) {
            let ip: u32 = ip_str.parse::<Ipv4Addr>()?.into();

            println!("[NETWORK] Blocking {}", ip_str);

            let _ = ip;
        }
    }

    Ok(())
}

fn parse_block_ip(message: &str) -> Option<&str> {
    let parts: Vec<&str> = message.trim().split_whitespace().collect();

    if parts.len() == 4 && parts[0] == "ACTION" && parts[1] == "network" && parts[2] == "BLOCK" {
        return Some(parts[3]);
    }

    None
}

async fn run_unix_server(
    shutdown_tx: broadcast::Sender<()>,
    mut shutdown: broadcast::Receiver<()>,
) -> Result<()> {
    if Path::new(UNIX_SOCKET).exists() {
        std::fs::remove_file(UNIX_SOCKET)?;
    }

    let listener = UnixListener::bind(UNIX_SOCKET)?;

    println!("Unix control socket ready {}", UNIX_SOCKET);

    loop {
        tokio::select! {

            _ = shutdown.recv() => {
                println!("Unix server shutting down");
                break;
            }

            result = listener.accept() => {

                let (stream, _) = result?;

                let shutdown_clone = shutdown_tx.clone();

                tokio::spawn(async move {
                    if let Err(e) = handle_unix(stream, shutdown_clone).await {
                        eprintln!("Unix handler error {:?}", e);
                    }
                });
            }
        }
    }

    Ok(())
}

async fn handle_unix(mut stream: UnixStream, shutdown_tx: broadcast::Sender<()>) -> Result<()> {
    let mut buffer = [0u8; 1024];

    let n = stream.read(&mut buffer).await?;
    let cmd = String::from_utf8_lossy(&buffer[..n]).trim().to_string();

    match cmd.as_str() {
        "-status" => {
            stream.write_all(b"Sentry running\n").await?;
        }

        "-stop" => {
            stream.write_all(b"Stopping sentry\n").await?;
            let _ = shutdown_tx.send(());
        }

        _ => {
            stream.write_all(b"Unknown command\n").await?;
        }
    }

    Ok(())
}
