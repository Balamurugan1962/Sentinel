use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct Config {
    pub client_id: String,
    pub root_ip: String,
    pub root_port: u16,
    pub heartbeat_interval: u64,
}
