use anyhow::Result;
use daemonize::Daemonize;
use std::collections::HashMap;
use std::fs::File;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use tokio::runtime::Builder;
use tokio::sync::{broadcast, mpsc, Mutex};

use crate::bridge::main::start_http;
use crate::config::{Config, SharedConfig};
use crate::tcp::run_tcp_server;

struct ClientMeta {
    tx: mpsc::Sender<String>,
    name: String,
    reg: String,
}

mod bridge;
mod config;
mod tcp;

type Clients = Arc<Mutex<HashMap<usize, ClientMeta>>>;
static CLIENT_COUNTER: AtomicUsize = AtomicUsize::new(1);

fn main() -> Result<()> {
    let config = Config::new();

    tracing_subscriber::fmt::init();

    if config.daemonize {
        daemonize(&config)?;
    }

    let config: SharedConfig = Arc::new(Mutex::new(config));

    let runtime = Builder::new_multi_thread().enable_all().build()?;
    runtime.block_on(async_main(config.clone()))
}

fn daemonize(config: &Config) -> Result<()> {
    let stdout = File::create(&config.stdout)?;
    let stderr = File::create(&config.stderr)?;

    let daemonize = Daemonize::new()
        .pid_file(&config.pid)
        .stdout(stdout)
        .stderr(stderr);

    daemonize.start()?;
    Ok(())
}

async fn async_main(config: SharedConfig) -> Result<()> {
    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));
    let (shutdown_tx, _) = broadcast::channel::<()>(1);

    let http_config = config.clone();
    let tcp_config = config.clone();

    let tcp_clients = clients.clone();
    let unix_clients = clients.clone();

    let tcp_shutdown = shutdown_tx.subscribe();
    let unix_shutdown = shutdown_tx.subscribe();

    tokio::spawn(async move {
        if let Err(e) = run_tcp_server(tcp_config, tcp_clients, tcp_shutdown).await {
            eprintln!("TCP server error: {:?}", e);
        }
    });

    let shutdown_tx_unix = shutdown_tx.clone();

    tokio::spawn(async move {
        if let Err(e) = start_http(http_config, unix_clients, shutdown_tx_unix, unix_shutdown).await
        {
            eprintln!("Unix server error: {:?}", e);
        }
    });

    shutdown_tx.subscribe().recv().await.ok();
    println!("Sentinel shutting down...");
    Ok(())
}
