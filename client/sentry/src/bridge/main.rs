use anyhow::Result;
use axum::{
    http::Method,
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use std::sync::Arc;
use tokio::{
    net::TcpListener,
    sync::{broadcast, mpsc},
};
use tower_http::cors::{Any, CorsLayer};

use crate::bridge::protocols::*;
use crate::{config::SharedConfig, user::SharedUser};

#[derive(Clone)]
pub struct AppState {
    pub shutdown_tx: broadcast::Sender<()>,
    pub user: SharedUser,
    pub config: SharedConfig,
    pub server_tx: mpsc::Sender<String>,
}

#[derive(Deserialize)]
pub struct InfoRequest {
    pub name: Option<String>,
    pub reg: Option<String>,
}

pub async fn run_http_server(
    shutdown_tx: broadcast::Sender<()>,
    mut shutdown: broadcast::Receiver<()>,
    user: SharedUser,
    config: SharedConfig,
    server_tx: mpsc::Sender<String>,
) -> Result<()> {
    let state = Arc::new(AppState {
        shutdown_tx,
        user,
        config,
        server_tx,
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any);

    let app = Router::new()
        .route("/info", post(info))
        .route("/logout", post(logout))
        .route("/status", get(status))
        .route("/stop", post(stop))
        .with_state(state)
        .layer(cors);

    let listener = TcpListener::bind("127.0.0.1:7373").await?;

    println!("HTTP control server ready at 127.0.0.1:7373");

    tokio::select! {

        res = axum::serve(listener, app) => {
            res?;
        }

        _ = shutdown.recv() => {
            println!("HTTP server shutting down");
        }
    }

    Ok(())
}
