use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub root: Node,
    pub nodes: Vec<Node>,
}

#[derive(Debug, Deserialize)]
pub struct Node {
    pub ip: String,
    pub port: String,
}

pub fn load_config(file: &str) -> Config {
    let content = fs::read_to_string(file).expect("Failed to read config file");

    return toml::from_str(&content).expect("Failed to parse TOML");
}
