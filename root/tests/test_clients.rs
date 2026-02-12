use std::io::{BufReader, Read, Write};
use std::net::{Shutdown, TcpStream};
use std::os::unix::net::UnixStream;
use std::process::{Child, Command};
use std::thread;
use std::time::Duration;

const SERVER_ADDR: &str = "127.0.0.1:19090";
const SOCKET_PATH: &str = "/tmp/sentinel.sock";

// ---------------------------------------------------------------------------

/// Start the server binary in foreground mode with the test config.
fn start_server() -> Child {
    let binary = env!("CARGO_BIN_EXE_sentinel");
    Command::new(binary)
        .args(["--no-daemon", "-c", "tests/test_config.toml"])
        .spawn()
        .expect("Failed to start server")
}

/// Connect a simulated node — sends the handshake `HELLO <id>\n`.
fn connect_node(id: u32) -> TcpStream {
    let mut stream = TcpStream::connect(SERVER_ADDR)
        .expect(&format!("Failed to connect node {}", id));
    let hello = format!("HELLO {}\n", id);
    stream
        .write_all(hello.as_bytes())
        .expect("Failed to send handshake");
    stream
}

/// Query connected nodes through the CLI unix socket.
fn query_nodes() -> String {
    let stream = UnixStream::connect(SOCKET_PATH).expect("Failed to connect CLI socket");
    let mut writer = stream.try_clone().unwrap();
    writeln!(writer, "ls").unwrap();
    let _ = writer.shutdown(Shutdown::Write);

    let mut response = String::new();
    BufReader::new(&stream)
        .read_to_string(&mut response)
        .unwrap();
    response
}

/// Small sleep to let the server process events.
fn wait() {
    thread::sleep(Duration::from_millis(500));
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn test_edge_cases() {
    // Clean up stale socket
    let _ = std::fs::remove_file(SOCKET_PATH);

    let mut server = start_server();
    // Give the server time to bind
    thread::sleep(Duration::from_secs(1));

    // ── Scenario 1: Both nodes connected ─────────────────────────────────
    println!("\n=== Scenario 1: Node 1 Connected, Node 2 Connected ===");
    let node1 = connect_node(1);
    let node2 = connect_node(2);
    wait();

    let result = query_nodes();
    println!("{}", result);
    assert!(result.contains("Node 1"), "Expected Node 1 in: {}", result);
    assert!(result.contains("Node 2"), "Expected Node 2 in: {}", result);

    // ── Scenario 2: Node 1 disconnected, Node 2 still connected ─────────
    println!("\n=== Scenario 2: Node 1 Disconnected, Node 2 Connected ===");
    drop(node1);
    wait();

    let result = query_nodes();
    println!("{}", result);
    assert!(
        !result.contains("Node 1"),
        "Node 1 should be gone in: {}",
        result
    );
    assert!(result.contains("Node 2"), "Expected Node 2 in: {}", result);

    // ── Scenario 3: Node 1 reconnected, Node 2 disconnected ─────────────
    println!("\n=== Scenario 3: Node 1 Connected, Node 2 Disconnected ===");
    let node1 = connect_node(1);
    drop(node2);
    wait();

    let result = query_nodes();
    println!("{}", result);
    assert!(result.contains("Node 1"), "Expected Node 1 in: {}", result);
    assert!(
        !result.contains("Node 2"),
        "Node 2 should be gone in: {}",
        result
    );

    // ── Scenario 4: Both disconnected ────────────────────────────────────
    println!("\n=== Scenario 4: Node 1 Disconnected, Node 2 Disconnected ===");
    drop(node1);
    wait();

    let result = query_nodes();
    println!("{}", result);
    assert!(
        result.contains("No nodes connected"),
        "Expected no nodes in: {}",
        result
    );

    // ── Cleanup ──────────────────────────────────────────────────────────
    server.kill().ok();
    server.wait().ok();
    let _ = std::fs::remove_file(SOCKET_PATH);
    println!("\n✅ All 4 edge-case scenarios passed!");
}
