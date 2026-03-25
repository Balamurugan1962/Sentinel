use anyhow::Result;
use axum::http::Method;
use axum::{extract::ws::WebSocketUpgrade, response::IntoResponse};
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};

use crate::config::SharedConfig;
use crate::Clients;

#[derive(Clone)]
pub struct AppState {
    pub config: SharedConfig,
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

#[derive(Deserialize)]
struct PolicyRequest {
    id: String,
    action: String,
    url: String,
}

pub fn extract_domain(url: &str) -> Option<String> {
    let url_to_parse = if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string()
    } else {
        format!("http://{}", url)
    };

    url::Url::parse(&url_to_parse)
        .ok()
        .and_then(|u| u.host_str().map(|host| host.to_string()))
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    let ip_addr = state.config.lock().await.server_ip.clone();

    ws.on_upgrade(|socket| async move {
        crate::bridge::kafka_ws::handle_ws(socket, ip_addr).await;
    })
}

pub async fn start_http(
    config: SharedConfig,
    clients: Clients,
    shutdown_tx: broadcast::Sender<()>,
    mut shutdown: broadcast::Receiver<()>,
) -> Result<()> {
    let state = AppState {
        config,
        clients,
        shutdown: shutdown_tx.clone(),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any);

    let app = Router::new()
        .route("/status", get(status))
        .route("/clients", get(list_clients))
        .route("/send", post(send_message))
        .route("/stop", post(stop_server))
        .route("/kafka/ws", get(ws_handler))
        .route("/policy", post(send_policy))
        .route("/allowed_sites/{id}", get(get_allowed_sites))
        .with_state(state)
        .layer(cors);

    let listener = TcpListener::bind("127.0.0.1:3737").await?;

    println!("[HTTP]: server listening on 127.0.0.1:3737");

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
                "register": meta.reg
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

async fn send_policy(
    State(state): State<AppState>,
    Json(req): Json<PolicyRequest>,
) -> Json<ApiResponse> {
    let domain = match extract_domain(&req.url) {
        Some(d) => d,
        None => return Json(ApiResponse { message: "Invalid URL".into() }),
    };

    let action = req.action.to_uppercase();
    if action != "ALLOW" && action != "BLOCK" {
        return Json(ApiResponse { message: "Invalid action (must be ALLOW or BLOCK)".into() });
    }

    let command = format!("ACTION network {} {}", action, domain);

    let mut guard = state.clients.lock().await;

    if req.id == "*" {
        for client in guard.values_mut() {
            if action == "ALLOW" {
                client.allowed_sites.insert(domain.clone());
            } else {
                client.allowed_sites.remove(&domain);
            }
            let _ = client.tx.send(command.clone()).await;
        }
        return Json(ApiResponse { message: "Broadcasted policy to all clients".into() });
    }

    if let Ok(id) = req.id.parse::<usize>() {
        if let Some(client) = guard.get_mut(&id) {
            if action == "ALLOW" {
                client.allowed_sites.insert(domain.clone());
            } else {
                client.allowed_sites.remove(&domain);
            }
            let _ = client.tx.send(command).await;
            return Json(ApiResponse { message: "Policy sent".into() });
        }
    }

    Json(ApiResponse { message: "Client not found".into() })
}

async fn get_allowed_sites(
    Path(id): Path<usize>,
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let guard = state.clients.lock().await;

    if let Some(client) = guard.get(&id) {
        let sites: Vec<String> = client.allowed_sites.iter().cloned().collect();
        Json(serde_json::json!({
            "id": id,
            "allowed_sites": sites
        }))
    } else {
        Json(serde_json::json!({ "error": "Client not found" }))
    }
}
