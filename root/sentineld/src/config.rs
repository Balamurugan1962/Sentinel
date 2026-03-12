use std::sync::Arc;

use tokio::sync::Mutex;

#[derive(Clone, Debug)]
pub struct Config {
    pub server_ip: String,

    pub stdout: String,
    pub stderr: String,
    pub pid: String,

    pub daemonize: bool,
    // pub verbose: bool,
}

impl Config {
    pub fn new() -> Config {
        Config {
            server_ip: "127.0.0.1".to_string(),

            stdout: "/tmp/sentinel.out".to_string(),
            stderr: "/tmp/sentinel.err".to_string(),
            pid: "/tmp/sentinel.out".to_string(),

            daemonize: true,
            // verbose: true,
        }
    }
}

pub type SharedConfig = Arc<Mutex<Config>>;
