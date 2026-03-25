use axum::extract::State;
use axum::routing::get;
use axum::{routing::post, Json, Router};
use rdkafka::producer::FutureRecord;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use tokio::{net::TcpListener, sync::broadcast};
use tower_http::cors::{Any, CorsLayer};

use crate::monitor::browser::init::AppState;

// TODO:
// need to share this to kafka
#[derive(Deserialize)]
struct Visit {
    url: String,
    timestamp: u64,
}

#[derive(Serialize)]
struct Decision {
    allowed: bool,
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

fn is_allowed(domain: &str, allowlist: &HashSet<String>) -> bool {
    allowlist
        .iter()
        .any(|allowed| domain == allowed || domain.ends_with(&format!(".{}", allowed)))
}

async fn firewall(State(state): State<Arc<AppState>>, Json(data): Json<Visit>) -> Json<Decision> {
    println!("Visited: {}", data.url);

    let domain = extract_domain(&data.url);

    let allowed = match domain {
        Some(ref d) => {
            let allowlist = state.allowlist.read().await;
            is_allowed(d, &allowlist)
        }
        None => false,
    };

    let payload = json!({
        "url": data.url,
        "timestamp": data.timestamp,
        "allowed": allowed
    })
    .to_string();

    let _ = state
        .producer
        .send(
            FutureRecord::to("27-browser")
                .payload(&payload)
                .key("visit"),
            Duration::from_secs(0),
        )
        .await;

    Json(Decision { allowed })
}

async fn get_allowlist(State(state): State<Arc<AppState>>) -> Json<Vec<String>> {
    let allowlist = state.allowlist.read().await;
    Json(allowlist.iter().cloned().collect())
}

pub async fn start_http(state: Arc<AppState>, _shutdown_tx: broadcast::Receiver<()>) {
    // before starting the system, need to enforce the
    // policy to the firefox, for this testing we need to do in ubuntu

    // enforce_policy();

    // TODO:
    // change cors allowed origins
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/firewall", post(firewall))
        .route("/allowlist", get(get_allowlist))
        .layer(cors)
        .with_state(state);

    // should always run in localhost
    // TODO:
    // need to make it secure. either by some kinda key or integrity check
    // but anyways its used to logs, well see.
    let listener = TcpListener::bind("127.0.0.1:7777").await.unwrap();

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
