use anyhow::Result;
use daemonize::Daemonize;
use std::{fs::File, sync::Arc};
use tokio::{
    runtime::Builder,
    sync::{broadcast, mpsc, Mutex},
};

use crate::{
    bridge::main::run_http_server,
    config::{Config, SharedConfig},
    monitor::init::start_monitor,
    tcp::root_server_task,
    user::{SharedUser, UserInfo},
};

mod bridge;
mod config;
mod monitor;
mod tcp;
mod user;

fn main() -> Result<()> {
    let config = Config::new();
    let verbose = config.verbose;
    tracing_subscriber::fmt::init();

    if config.daemonize {
        daemonize(&config)?;
    }

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
    let shutdown_http = shutdown_tx.subscribe();

    tokio::spawn(root_server_task(
        network_tx,
        server_rx,
        user.clone(),
        config.clone(),
        shutdown_root,
    ));

    tokio::spawn(start_monitor(network_rx, shutdown_tx.subscribe()));

    run_http_server(
        shutdown_tx,
        shutdown_http,
        user.clone(),
        config.clone(),
        server_tx,
    )
    .await?;

    Ok(())
}
