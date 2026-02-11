use serde::Deserialize;
use std::{fs, net::IpAddr, net::SocketAddr};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub root: Node,
    pub nodes: Vec<Node>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Node {
    pub ip: IpAddr,
    pub port: u16,
}

impl Node {
    pub fn socket_addr(&self) -> SocketAddr {
        SocketAddr::new(self.ip, self.port)
    }
}

pub fn load_config(file: &str) -> Config {
    let content = fs::read_to_string(file).expect("Failed to read config file");

    return toml::from_str(&content).expect("Failed to parse TOML");
}
