use anyhow::Result;
use axum::{extract::ws::WebSocketUpgrade, response::IntoResponse};
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tokio::sync::broadcast;

use crate::Clients;

#[derive(Clone)]
struct AppState {
    clients: Clients,
    shutdown: broadcast::Sender<()>,
}

#[derive(Serialize)]
struct ApiResponse {
    message: String,
}

#[derive(Deserialize)]
struct SendRequest {
    id: usize,
    message: String,
}

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(|socket| async move {
        crate::bridge::kafka_ws::handle_ws(socket).await;
    })
}

pub async fn start_http(
    clients: Clients,
    shutdown_tx: broadcast::Sender<()>,
    mut shutdown: broadcast::Receiver<()>,
) -> Result<()> {
    let state = AppState {
        clients,
        shutdown: shutdown_tx.clone(),
    };

    let app = Router::new()
        .route("/status", get(status))
        .route("/clients", get(list_clients))
        .route("/send", post(send_message))
        .route("/stop", post(stop_server))
        .route("/kafka/ws", get(ws_handler))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:3737").await?;

    println!("HTTP server listening on 127.0.0.1:3737");

    tokio::select! {

        result = axum::serve(listener, app) => {
            result?;
        }

        _ = shutdown.recv() => {
            println!("HTTP server shutting down...");
        }
    }

    Ok(())
}

async fn status() -> Json<ApiResponse> {
    Json(ApiResponse {
        message: "Status: Active".into(),
    })
}

async fn list_clients(State(state): State<AppState>) -> Json<serde_json::Value> {
    let guard = state.clients.lock().await;

    let list: Vec<_> = guard
        .iter()
        .map(|(id, meta)| {
            serde_json::json!({
                "id": id,
                "name": meta.name,
                "reg": meta.reg
            })
        })
        .collect();

    Json(serde_json::json!(list))
}

async fn send_message(
    State(state): State<AppState>,
    Json(req): Json<SendRequest>,
) -> Json<ApiResponse> {
    let guard = state.clients.lock().await;

    if let Some(client) = guard.get(&req.id) {
        let _ = client.tx.send(req.message.clone()).await;

        Json(ApiResponse {
            message: "Message sent".into(),
        })
    } else {
        Json(ApiResponse {
            message: "Client not found".into(),
        })
    }
}

async fn stop_server(State(state): State<AppState>) -> Json<ApiResponse> {
    let _ = state.shutdown.send(());

    Json(ApiResponse {
        message: "Stopping Sentinel".into(),
    })
}
