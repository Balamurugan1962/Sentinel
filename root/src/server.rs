use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

pub fn start_server(addr: &str) {
    let listener = TcpListener::bind(addr).expect("Failed to bind root server");

    println!("Root listening on {}", addr);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New node connected!");
                // Create seperate thread and send it there.
                thread::spawn(|| {
                    handle_node(stream);
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }
}

fn handle_node(mut stream: TcpStream) {
    let mut buffer = [0; 512];

    match stream.read(&mut buffer) {
        Ok(size) => {
            println!("Received: {}", String::from_utf8_lossy(&buffer[..size]));

            stream.write_all(b"ACK from root").unwrap();
        }
        Err(e) => {
            eprintln!("Read failed: {}", e);
        }
    }
}
