use std::{net::UdpSocket, sync::Arc};

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

fn get_local_ip() -> std::io::Result<String> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;

    // Connect to an external address (nothing is actually sent)
    // 192.0.2.1:80 is a test net reserved, so always exists.
    socket.connect("192.0.2.1:80")?;

    let local_addr = socket.local_addr()?;

    Ok(local_addr.ip().to_string())
}

impl Config {
    pub fn new() -> Config {
        let mut server_ip = "127.0.0.1".to_string();
        match get_local_ip() {
            Ok(ip) => server_ip = ip,
            Err(e) => println!("Error: {}", e),
        }
        Config {
            server_ip: server_ip,

            stdout: "/tmp/sentinel.out".to_string(),
            stderr: "/tmp/sentinel.err".to_string(),
            pid: "/tmp/sentinel.out".to_string(),

            daemonize: false,
            // verbose: true,
        }
    }
}

pub type SharedConfig = Arc<Mutex<Config>>;
