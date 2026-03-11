use axum::extract::ws::{Message as WsMessage, WebSocket};
use futures_util::StreamExt;
use rdkafka::message::Message;
use rdkafka::{
    config::ClientConfig,
    consumer::{Consumer, StreamConsumer},
};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
struct SubscribeRequest {
    topic: String,
}

pub async fn handle_ws(mut socket: WebSocket) {
    println!("WebSocket connected");

    let msg = match socket.next().await {
        Some(Ok(WsMessage::Text(m))) => m,
        _ => {
            println!("Invalid subscribe message");
            return;
        }
    };

    let sub: SubscribeRequest = match serde_json::from_str(&msg) {
        Ok(v) => v,
        Err(e) => {
            println!("Invalid JSON: {:?}", e);
            return;
        }
    };

    println!("Subscribing to topic: {}", sub.topic);

    let group_id = format!("sentinel-ws-{}", Uuid::new_v4());

    let consumer: StreamConsumer = match ClientConfig::new()
        .set("bootstrap.servers", "localhost:9092")
        .set("group.id", group_id)
        .set("auto.offset.reset", "earliest")
        .create()
    {
        Ok(c) => c,
        Err(e) => {
            println!("Kafka consumer error: {:?}", e);
            return;
        }
    };

    if let Err(e) = consumer.subscribe(&[&sub.topic]) {
        println!("Kafka subscribe error: {:?}", e);
        return;
    }

    let mut stream = consumer.stream();

    while let Some(message) = stream.next().await {
        match message {
            Ok(m) => {
                if let Some(payload) = m.payload() {
                    let text = String::from_utf8_lossy(payload);

                    if socket
                        .send(WsMessage::Text(text.to_string().into()))
                        .await
                        .is_err()
                    {
                        println!("WebSocket disconnected");
                        break;
                    }
                }
            }

            Err(e) => {
                println!("Kafka error: {:?}", e);
            }
        }
    }
}
