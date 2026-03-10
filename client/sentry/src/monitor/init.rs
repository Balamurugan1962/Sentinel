use tokio::sync::{broadcast, mpsc};

use crate::monitor::{browser::browser_monitor, network::network_task};

pub async fn start_monitor(
    network_rx: mpsc::Receiver<String>,
    shutdown_tx: broadcast::Receiver<()>,
) {
    println!("[SENTRY] Starting Monitor!");

    tokio::spawn(browser_monitor(shutdown_tx.resubscribe()));
    tokio::spawn(network_task(network_rx, shutdown_tx.resubscribe()));
}
