use anyhow::Result;
use daemonize::Daemonize;
use std::{fs::File, net::Ipv4Addr, sync::Arc};
use tokio::{
    runtime::Builder,
    sync::{broadcast, mpsc, Mutex},
};

use crate::{
    config::{Config, SharedConfig},
    tcp::root_server_task,
    unix::run_unix_server,
    user::{SharedUser, UserInfo},
};

mod config;
mod tcp;
mod unix;
mod user;

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

    let user: SharedUser = Arc::new(Mutex::new(UserInfo::new()));

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
