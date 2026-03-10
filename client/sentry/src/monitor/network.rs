use anyhow::Result;
use std::net::Ipv4Addr;
use tokio::sync::{broadcast, mpsc};

// TODO:
// need to have three async functions
// 1. mpsc listener for action protocol
// 2. DNS proxy server firewall
// 3. eBPF kernel level firewall (need to decide weather to add)
pub async fn network_task(
    mut rx: mpsc::Receiver<String>,
    _shutdown_tx: broadcast::Receiver<()>,
) -> Result<()> {
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
