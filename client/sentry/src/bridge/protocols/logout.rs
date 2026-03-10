use std::sync::Arc;

use axum::extract::State;

use crate::bridge::main::AppState;

pub async fn logout(State(state): State<Arc<AppState>>) -> Result<String, String> {
    let mut u = state.user.lock().await;

    u.name = "unknown".to_string();
    u.reg = "unknown".to_string();

    let version = state.config.lock().await.version.clone();

    let info = format!(
        "INFO user {}\n{{\"name\":\"{}\",\"regno\":\"{}\"}}\n",
        version, u.name, u.reg
    );

    state
        .server_tx
        .send(info)
        .await
        .map_err(|e| e.to_string())?;

    Ok("Info updated".to_string())
}
