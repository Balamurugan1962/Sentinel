use anyhow::Result;
use std::{net::Ipv4Addr, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::mpsc,
    time::sleep,
};

//TODO:
// should get it from config
const CLIENT_ID: &str = "3122235001027";
const VERSION: &str = "SNT0.1";

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<()> {
    println!("Sentinel Client Starting...");

    // Channel between server thread and network logger
    let (tx, rx) = mpsc::channel::<String>(100);

    // Spawn root server polling thread
    let server_task = tokio::spawn(root_server_task(tx));

    // Spawn network logger thread
    let network_task = tokio::spawn(network_logger_task(rx));

    tokio::try_join!(server_task, network_task)?;

    Ok(())
}

async fn root_server_task(tx: mpsc::Sender<String>) -> Result<()> {
    println!("[SERVER THREAD] Connecting to root server...");

    let mut stream = TcpStream::connect("127.0.0.1:1612").await?;

    // ---- Handshake ----
    let hello = format!("HELLO {} {}\n", CLIENT_ID, VERSION);
    stream.write_all(hello.as_bytes()).await?;

    let mut buffer = [0u8; 1024];
    let n = stream.read(&mut buffer).await?;
    let response = String::from_utf8_lossy(&buffer[..n]);

    if response.trim() != "AKN" {
        println!("Handshake failed");
        return Ok(());
    }

    println!("[SERVER THREAD] Handshake successful");

    // ---- Polling Loop ----
    loop {
        let n = stream.read(&mut buffer).await?;

        if n == 0 {
            println!("Server disconnected");
            break;
        }

        let message = String::from_utf8_lossy(&buffer[..n]).to_string();

        if message.contains("ACTION self") {
            println!("[SERVER THREAD] {}", message.trim());
        } else if message.contains("ACTION network") {
            tx.send(message.clone()).await?;
        }

        sleep(Duration::from_secs(2)).await;
    }

    Ok(())
}

// TODO:
// Need to refine this part to support
// Kafka logging.
async fn network_logger_task(mut rx: mpsc::Receiver<String>) -> Result<()> {
    println!("[NETWORK THREAD] Loading XDP firewall...");
    //TODO:
    // Test this part
    // ip link show dev enp3s0
    // Before
    // prog/xdp id 52
    // After
    // ip link show dev enp3s0

    // let mut bpf = aya::Ebpf::load(aya::include_bytes_aligned!(concat!(
    //     env!("OUT_DIR"),
    //     "/xdp-firewall"
    // )))?;

    // let program: &mut Xdp = bpf.program_mut("xdp_firewall")?.try_into()?;

    // program.load()?;
    // program.attach("enp3s0", XdpFlags::default())?;

    println!("[NETWORK THREAD] Firewall attached");

    // Get BLOCKLIST map
    // let mut blocklist: HashMap<_, u32, u32> = HashMap::try_from(bpf.map_mut("BLOCKLIST")?)?;

    println!("[NETWORK THREAD] Ready for actions...");

    while let Some(message) = rx.recv().await {
        println!("[NETWORK THREAD] Received: {}", message.trim());

        // Expected format:
        // ACTION network BLOCK 8.8.8.8
        if let Some(ip_str) = parse_block_ip(&message) {
            let ip: u32 = ip_str.parse::<Ipv4Addr>()?.into();

            // blocklist.insert(ip, 0, 0)?;

            println!("[NETWORK THREAD] Blocked IP {}", ip_str);
        }
    }

    Ok(())
}

fn parse_block_ip(message: &str) -> Option<&str> {
    let parts: Vec<&str> = message.trim().split_whitespace().collect();

    if parts.len() == 4 && parts[0] == "ACTION" && parts[1] == "network" && parts[2] == "BLOCK" {
        return Some(parts[3]);
    }

    None
}
