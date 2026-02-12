use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::{Arc, Mutex};
use std::thread;

use chrono::Local;

use crate::config::{Config, Node};

pub const SOCKET_PATH: &str = "/tmp/sentinel.sock";

type ActiveNodes = Arc<Mutex<HashMap<u32, NodeInfo>>>;

#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub id: u32,
    pub ip: String,
    pub connected_at: String,
}

fn log_to_file(target: &str, message: &str) {
    let path = format!("logs/{}.log", target);
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) {
        let ts = Local::now().format("%Y-%m-%d %H:%M:%S");
        let _ = writeln!(file, "[{}] {}", ts, message);
    }
}

/// Read a single line by consuming one byte at a time.
/// Avoids BufReader so subsequent raw reads on the same stream are unaffected.
fn read_handshake_line(stream: &mut TcpStream) -> Option<String> {
    let mut buf = Vec::with_capacity(64);
    let mut byte = [0u8; 1];
    loop {
        match stream.read(&mut byte) {
            Ok(0) => return None,
            Ok(_) if byte[0] == b'\n' => break,
            Ok(_) => buf.push(byte[0]),
            Err(_) => return None,
        }
    }
    String::from_utf8(buf).ok()
}

pub fn start_server(addr: &str, config: Config) {
    fs::create_dir_all("logs").ok();

    let active_nodes: ActiveNodes = Arc::new(Mutex::new(HashMap::new()));
    let config_nodes = Arc::new(config.nodes);

    // CLI socket runs in its own thread so it can serve queries while the TCP
    // listener blocks on incoming node connections.
    let cli_nodes = Arc::clone(&active_nodes);
    thread::spawn(move || run_cli_socket(cli_nodes));

    let listener = TcpListener::bind(addr).expect("Failed to bind root server");
    log_to_file("sentinel", &format!("Root listening on {}", addr));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let nodes = Arc::clone(&active_nodes);
                let cfg = Arc::clone(&config_nodes);
                thread::spawn(move || handle_node(stream, nodes, cfg));
            }
            Err(e) => log_to_file("sentinel", &format!("Accept error: {}", e)),
        }
    }
}

// ─── Node handling ──────────────────────────────────────────────────────────

fn handle_node(mut stream: TcpStream, active_nodes: ActiveNodes, config_nodes: Arc<Vec<Node>>) {
    let peer_ip = match stream.peer_addr() {
        Ok(addr) => addr.ip(),
        Err(_) => return,
    };

    // Reject IPs not present in the config
    if !config_nodes.iter().any(|n| n.ip == peer_ip) {
        log_to_file("unknown", &format!("Rejected unknown IP: {}", peer_ip));
        let _ = stream.shutdown(std::net::Shutdown::Both);
        return;
    }

    // Handshake: client sends "HELLO <node_id>\n"
    let node_id = match parse_handshake(&mut stream, &peer_ip) {
        Some(id) => id,
        None => return,
    };

    // Verify node ID + IP combination exists in config
    if !config_nodes.iter().any(|n| n.id == node_id && n.ip == peer_ip) {
        log_to_file("unknown", &format!("Node {} from {} not in config", node_id, peer_ip));
        let _ = stream.shutdown(std::net::Shutdown::Both);
        return;
    }

    let log_target = node_id.to_string();

    register_node(&active_nodes, node_id, &peer_ip.to_string(), &log_target);
    read_loop(&mut stream, node_id, &log_target);
    unregister_node(&active_nodes, node_id, &log_target);
}

fn parse_handshake(stream: &mut TcpStream, peer_ip: &std::net::IpAddr) -> Option<u32> {
    let line = read_handshake_line(stream).or_else(|| {
        log_to_file("unknown", &format!("Handshake read failed from {}", peer_ip));
        None
    })?;

    let parts: Vec<&str> = line.trim().split_whitespace().collect();
    if parts.len() != 2 || parts[0] != "HELLO" {
        log_to_file("unknown", &format!("Malformed handshake from {}: {:?}", peer_ip, line));
        let _ = stream.shutdown(std::net::Shutdown::Both);
        return None;
    }

    parts[1].parse::<u32>().ok().or_else(|| {
        log_to_file("unknown", &format!("Invalid node ID from {}: {}", peer_ip, parts[1]));
        None
    })
}

fn register_node(active_nodes: &ActiveNodes, id: u32, ip: &str, log_target: &str) {
    let mut map = active_nodes.lock().unwrap();

    if map.contains_key(&id) {
        log_to_file(log_target, &format!("Node {} reconnected, replacing old session", id));
        map.remove(&id);
    }

    map.insert(id, NodeInfo {
        id,
        ip: ip.to_string(),
        connected_at: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    });

    log_to_file(log_target, &format!("Node {} connected (active: {})", id, map.len()));
}

fn read_loop(stream: &mut TcpStream, node_id: u32, log_target: &str) {
    let mut buffer = [0u8; 512];
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => {
                log_to_file(log_target, &format!("Node {} disconnected", node_id));
                break;
            }
            Ok(n) => {
                let msg = String::from_utf8_lossy(&buffer[..n]);
                log_to_file(log_target, &format!("Node {} says: {}", node_id, msg));
            }
            Err(e) => {
                log_to_file(log_target, &format!("Node {} read error: {}", node_id, e));
                break;
            }
        }
    }
}

fn unregister_node(active_nodes: &ActiveNodes, id: u32, log_target: &str) {
    let mut map = active_nodes.lock().unwrap();
    map.remove(&id);
    log_to_file(log_target, &format!("Node {} removed (active: {})", id, map.len()));
}

// ─── CLI socket ─────────────────────────────────────────────────────────────

fn run_cli_socket(active_nodes: ActiveNodes) {
    let _ = fs::remove_file(SOCKET_PATH);
    let listener = UnixListener::bind(SOCKET_PATH).expect("Failed to bind CLI socket");
    log_to_file("sentinel", &format!("CLI socket listening on {}", SOCKET_PATH));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let nodes = Arc::clone(&active_nodes);
                thread::spawn(move || handle_cli_client(stream, nodes));
            }
            Err(e) => log_to_file("sentinel", &format!("CLI socket error: {}", e)),
        }
    }
}

fn handle_cli_client(stream: UnixStream, active_nodes: ActiveNodes) {
    let mut reader = BufReader::new(&stream);
    let mut writer = stream.try_clone().unwrap();
    let mut line = String::new();

    if reader.read_line(&mut line).is_ok() {
        let cmd = line.trim();
        match cmd {
            "ls" => {
                let map = active_nodes.lock().unwrap();
                if map.is_empty() {
                    let _ = writeln!(writer, "No nodes connected.");
                } else {
                    let _ = writeln!(writer, "Connected Nodes:");
                    let mut ids: Vec<&u32> = map.keys().collect();
                    ids.sort();
                    for id in ids {
                        let info = &map[id];
                        let _ = writeln!(
                            writer,
                            "  Node {} — {} (connected since {})",
                            info.id, info.ip, info.connected_at
                        );
                    }
                }
            }
            "shutdown" => {
                log_to_file("sentinel", "Shutdown requested via CLI");
                let _ = writeln!(writer, "Sentinel server shutting down...");
                // Clean up socket and exit
                let _ = fs::remove_file(SOCKET_PATH);
                std::process::exit(0);
            }
            _ => {
                let _ = writeln!(writer, "Unknown command: {}", cmd);
            }
        }
    }
}
