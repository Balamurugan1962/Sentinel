use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt};
use chrono::Local;
use serde_json::Value;

#[tokio::main]
async fn main() {

    let listener = TcpListener::bind("0.0.0.0:9000")
        .await
        .expect("Failed to bind port");

    println!("Sentinel Root Server Running on port 9000");

    loop {

        let (mut socket, addr) = listener.accept()
            .await
            .expect("Failed to accept connection");

        tokio::spawn(async move {

            let mut buffer = vec![0; 2048];

            match socket.read(&mut buffer).await {

                Ok(n) if n > 0 => {

                    let message = String::from_utf8_lossy(&buffer[..n]);

                    if let Ok(json) = serde_json::from_str::<Value>(&message) {

                        let timestamp = Local::now()
                            .format("%Y-%m-%d %H:%M:%S");

                        println!(
                            "[{}] {} -> {}",
                            timestamp,
                            addr,
                            json
                        );
                    }
                }

                _ => {}
            }
        });
    }
}
