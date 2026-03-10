use axum::extract::State;
use axum::{routing::post, Json, Router};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;
use serde::Deserialize;
use serde_json::json;
use std::io::Write;
use std::sync::Arc;
use std::time::Duration;
use std::{fs, path::Path};
use tokio::{net::TcpListener, sync::broadcast};
use tower_http::cors::{Any, CorsLayer};

// TODO:
// need to share this to kafka
#[derive(Deserialize)]
struct Visit {
    url: String,
    timestamp: u64,
}

struct AppState {
    producer: FutureProducer,
}

fn create_kafka_producer(kafka_ip: &str) -> FutureProducer {
    ClientConfig::new()
        .set("bootstrap.servers", kafka_ip)
        .create()
        .expect("Failed to create Kafka producer")
}

// Need to test and ensure this
fn _enforce_firefox_policy() {
    if !Path::new("/usr/bin/firefox").exists() {
        println!("Firefox not installed, skipping policy.");
        return;
    }

    let dir = "/etc/firefox/policies";
    let path = format!("{}/policies.json", dir);

    fs::create_dir_all(dir).unwrap();

    let policy = r#"
{
  "policies": {
    "DisableDeveloperTools": true,
    "DNSOverHTTPS": { "Enabled": false },
    "Extensions": {
      "Install": [
        "https://your-server/monitor.xpi"
      ],
      "Locked": [
        "monitor@sentinel"
      ]
    }
  }
}
"#;

    let mut file = fs::File::create(path).unwrap();
    file.write_all(policy.as_bytes()).unwrap();

    println!("Firefox policy enforced");
}

fn _enforce_policy() {
    #[cfg(target_os = "linux")]
    _enforce_firefox_policy();
}

async fn log(State(state): State<Arc<AppState>>, Json(data): Json<Visit>) {
    println!("Visited: {}", data.url);

    let payload = json!({
        "url": data.url,
        "timestamp": data.timestamp
    })
    .to_string();

    let _ = state
        .producer
        .send(
            // Topic: [Client_id]-log-browser
            FutureRecord::to("[client_id]-browser")
                .payload(&payload)
                .key("visit"),
            Duration::from_secs(0),
        )
        .await;
}

pub async fn browser_monitor(_shutdown_tx: broadcast::Receiver<()>) {
    // TODO:
    // need to hook it with config

    let kafka_ip = "127.0.0.1";
    let producer = create_kafka_producer(kafka_ip);

    let state = Arc::new(AppState { producer });

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
        .route("/log", post(log))
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
