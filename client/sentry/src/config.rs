use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct Config {
    pub client_id: String,
    pub version: String,

    pub server_ip: String,
    pub server_port: String,

    pub unix_socket: String,

    pub stdout: String,
    pub stderr: String,
    pub pid: String,

    pub verbose: bool,

    pub daemonize: bool,
}

impl Config {
    pub fn new() -> Config {
        Config {
            client_id: "27".to_string(),
            version: "0.1".to_string(),

            server_ip: "127.0.0.1".to_string(),
            server_port: "1612".to_string(),

            unix_socket: "/tmp/sentry.sock".to_string(),

            stdout: "/tmp/sentry.out".to_string(),
            stderr: "/tmp/sentry.err".to_string(),
            pid: "/tmp/sentry.pid".to_string(),

            verbose: true,

            daemonize: false,
        }
    }
}

pub type SharedConfig = Arc<Mutex<Config>>;
