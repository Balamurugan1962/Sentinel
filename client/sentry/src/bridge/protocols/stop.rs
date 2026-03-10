use std::sync::Arc;

use axum::extract::State;

use crate::bridge::main::AppState;

pub async fn stop(State(state): State<Arc<AppState>>) -> &'static str {
    let _ = state.shutdown_tx.send(());

    "Stopping sentry"
}
