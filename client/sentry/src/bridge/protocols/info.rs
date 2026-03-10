use std::sync::Arc;

use axum::{extract::State, Json};

use crate::bridge::main::{AppState, InfoRequest};

pub async fn info(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<InfoRequest>,
) -> Result<String, String> {
    let mut u = state.user.lock().await;

    if let Some(name) = payload.name {
        u.name = name;
    }

    if let Some(reg) = payload.reg {
        u.reg = reg;
    }

    let version = state.config.lock().await.version.clone();

    let info = format!(
        "INFO user {}\n{{\"name\":\"{}\",\"regno\":\"{}\"}}\n",
        version, u.name, u.reg
    );

    if let Err(e) = state.server_tx.send(info).await {
        eprintln!("server channel closed: {}", e);
    }

    Ok("Info updated".to_string())
}
