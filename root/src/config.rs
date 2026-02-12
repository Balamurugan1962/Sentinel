use serde::Deserialize;
use std::fmt;
use std::fs;
use std::net::IpAddr;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub root: Root,
    pub nodes: Vec<Node>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Root {
    pub ip: IpAddr,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Node {
    pub id: u32,
    pub ip: IpAddr,
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Root Server:")?;
        writeln!(f, "  {}:{}", self.root.ip, self.root.port)?;
        writeln!(f, "")?;
        writeln!(f, "Nodes ({}):", self.nodes.len())?;
        for node in &self.nodes {
            writeln!(f, "  Node {} — {}", node.id, node.ip)?;
        }
        Ok(())
    }
}

pub fn load_config(path: &str) -> Config {
    let content = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Failed to read config '{}': {}", path, e));

    toml::from_str(&content)
        .unwrap_or_else(|e| panic!("Failed to parse config '{}': {}", path, e))
}
