use anyhow::Result;
use daemonize::Daemonize;
use std::{fs::File, net::Ipv4Addr, path::Path, sync::Arc, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpStream, UnixListener, UnixStream},
    runtime::Builder,
    sync::{broadcast, mpsc, Mutex},
    time::sleep,
};

use crate::config::{Config, SharedConfig};

mod config;

#[derive(Default)]
struct UserInfo {
    name: String,
    reg: String,
}

type SharedUser = Arc<Mutex<UserInfo>>;

fn main() -> Result<()> {
    let config = Config::new();
    let verbose = config.verbose;

    if verbose {
        println!("[SENTRY] Start state started!");
    }

    tracing_subscriber::fmt::init();

    daemonize(&config)?;

    if verbose {
        println!("[SENTRY] Daemon Started!");
    }

    let config: SharedConfig = Arc::new(Mutex::new(config));

    let runtime = Builder::new_multi_thread().enable_all().build()?;
    runtime.block_on(async_main(config.clone()))
}

fn daemonize(config: &Config) -> Result<()> {
    let stdout = File::create(&config.stdout)?;
    let stderr = File::create(&config.stderr)?;

    let daemon = Daemonize::new()
        .pid_file(&config.pid)
        .stdout(stdout)
        .stderr(stderr);

    daemon.start()?;
    Ok(())
}

async fn async_main(config: SharedConfig) -> Result<()> {
    let verbose = config.lock().await.verbose.clone();

    if verbose {
        println!("[SENTRY] Sentry daemon starting");
    }

    let user: SharedUser = Arc::new(Mutex::new(UserInfo {
        name: "unknown".into(),
        reg: "unknown".into(),
    }));

    let (network_tx, network_rx) = mpsc::channel::<String>(100);
    let (server_tx, server_rx) = mpsc::channel::<String>(100);

    let (shutdown_tx, _) = broadcast::channel::<()>(1);

    let shutdown_root = shutdown_tx.subscribe();
    let shutdown_unix = shutdown_tx.subscribe();

    tokio::spawn(root_server_task(
        network_tx,
        server_rx,
        user.clone(),
        config.clone(),
        shutdown_root,
    ));

    tokio::spawn(network_logger_task(network_rx));

    run_unix_server(
        shutdown_tx,
        shutdown_unix,
        user.clone(),
        config.clone(),
        server_tx,
    )
    .await?;

    Ok(())
}

async fn root_server_task(
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
