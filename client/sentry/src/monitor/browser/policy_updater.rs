use std::sync::Arc;

use anyhow::Result;
use tokio::sync::mpsc;

use crate::monitor::browser::{browser::extract_domain, init::AppState};

enum Action<'a> {
    Allow(&'a str),
    Block(&'a str),
}

fn parse_action(message: &str) -> Option<Action<'_>> {
    let parts: Vec<&str> = message.trim().split_whitespace().collect();

    if parts.len() != 4 || parts[0] != "ACTION" || parts[1] != "network" {
        return None;
    }

    match parts[2] {
        "ALLOW" => Some(Action::Allow(parts[3])),
        "BLOCK" => Some(Action::Block(parts[3])),
        _ => None,
    }
}

pub async fn policy_updater(mut rx: mpsc::Receiver<String>, state: Arc<AppState>) -> Result<()> {
    println!("[POLICY] Updater started");

    while let Some(message) = rx.recv().await {
        println!("[POLICY] {}", message);

        match parse_action(&message) {
            Some(Action::Allow(url)) => {
                if let Some(domain) = extract_domain(url) {
                    let mut allowlist = state.allowlist.write().await;
                    allowlist.insert(domain.clone());
                    println!("[POLICY] Allowed {}", domain);
                }
            }

            Some(Action::Block(url)) => {
                if let Some(domain) = extract_domain(url) {
                    let mut allowlist = state.allowlist.write().await;
                    allowlist.remove(&domain);
                    println!("[POLICY] Blocked {}", domain);
                }
            }

            None => println!("[POLICY] Invalid command"),
        }
    }

    Ok(())
}
