use tokio::sync::{broadcast, mpsc};

use crate::monitor::browser::init::browser_monitor;

pub async fn start_monitor(
    network_rx: mpsc::Receiver<String>,
    shutdown_tx: broadcast::Receiver<()>,
) {
    println!("[SENTRY] Starting Monitor!");

    tokio::spawn(browser_monitor(network_rx, shutdown_tx.resubscribe()));
}
