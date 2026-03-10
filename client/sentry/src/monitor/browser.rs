use axum::{routing::post, Json, Router};
use serde::Deserialize;
use tokio::{net::TcpListener, sync::broadcast};

// TODO:
// need to share this to kafka
#[derive(Deserialize)]
struct Visit {
    url: String,
    _timestamp: u64,
}

fn enforce_policy() {} // need to implement policy enfocement in firefox

async fn log(Json(data): Json<Visit>) {
    println!("Visited: {}", data.url); // need to send this log to kafka
}

pub async fn browser_monitor(_shutdown_tx: broadcast::Receiver<()>) {
    // before starting the system, need to enforce the
    // policy to the firefox, for this testing we need to do in ubuntu

    let app = Router::new().route("/log", post(log));
    // should always run in localhost
    // TODO:
    // need to make it secure. either by some kinda key or integrity check
    // but anyways its used to logs, well see.
    let listener = TcpListener::bind("127.0.0.1:7777").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
