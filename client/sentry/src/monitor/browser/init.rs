use std::{collections::HashSet, sync::Arc, time::Duration};

use rdkafka::{
    config::RDKafkaLogLevel,
    producer::{FutureProducer, Producer},
    ClientConfig,
};
use tokio::{
    sync::{broadcast, mpsc, RwLock},
    time::sleep,
};

use crate::monitor::browser::{browser::start_http, policy_updater::policy_updater};

pub struct AppState {
    pub producer: FutureProducer,
    pub allowlist: RwLock<HashSet<String>>,
}

async fn create_kafka_producer(kafka_ip: &str) -> FutureProducer {
    loop {
        match ClientConfig::new()
            .set("bootstrap.servers", kafka_ip)
            .set("log_level", "0")
            .set_log_level(RDKafkaLogLevel::Emerg)
            .create::<FutureProducer>()
        {
            Ok(producer) => {
                match producer
                    .client()
                    .fetch_metadata(None, Duration::from_secs(2))
                {
                    Ok(_) => {
                        println!("[SENTRY] Kafka connected");
                        return producer;
                    }
                    Err(_e) => {
                        println!("[KAFKA]: Kafka not ready. retrying...");
                    }
                }
            }
            Err(_e) => {
                println!("[KAFKA]: Failed creating producer");
            }
        }
        sleep(Duration::from_secs(10)).await;
    }
}

pub async fn browser_monitor(rx: mpsc::Receiver<String>, shutdown_tx: broadcast::Receiver<()>) {
    let kafka_ip = "127.0.0.1:9092";
    let producer = create_kafka_producer(kafka_ip).await;

    let allowlist = RwLock::new(
        vec!["lms.ssn.edu.in"]
            .into_iter()
            .map(String::from)
            .collect(),
    );

    let state = Arc::new(AppState {
        producer,
        allowlist,
    });

    tokio::spawn(policy_updater(rx, state.clone()));

    tokio::spawn(start_http(state.clone(), shutdown_tx.resubscribe()));
}
