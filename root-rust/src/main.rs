use tokio::net::TcpListener;
use tokio::io::AsyncReadExt;

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

            let mut buffer = [0u8; 1024];

            if let Ok(n) = socket.read(&mut buffer).await {
                if n > 0 {
                    let message = String::from_utf8_lossy(&buffer[..n]);
                    println!("{}",message.trim());
                }
            }
        });
    }
}
