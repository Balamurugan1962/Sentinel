use std::env;
use std::io::{BufRead, BufReader, Write};
use std::net::{Shutdown, TcpListener};
use std::os::unix::net::UnixStream;

use daemonize::Daemonize;

use crate::config::load_config;
use crate::server::{SOCKET_PATH, start_server};

mod config;
mod server;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.contains(&"-h".to_string()) || args.contains(&"--help".to_string()) {
        print_help();
        return;
    }

    if args.contains(&"-ls".to_string()) {
        cli_send_command("ls");
        return;
    }

    if args.contains(&"--shutdown".to_string()) {
        println!("Shutting down Sentinel...");
        cli_send_command("shutdown");
        println!("Sentinel is shutdown.");
        return;
    }

    // Parse flags that need a config
    let mut config_path = "config.toml".to_string();
    let mut no_daemon = false;
    let mut print_config = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-c" | "--config" => {
                i += 1;
                if i < args.len() {
                    config_path = args[i].clone();
                }
            }
            "--no-daemon" => no_daemon = true,
            "--config-print" => print_config = true,
            _ => {
                eprintln!("Error: Unknown argument '{}'\n", args[i]);
                print_help();
                return;
            }
        }
        i += 1;
    }

    let config = load_config(&config_path);

    if print_config {
        println!("Sentinel — Loaded config from '{}'\n", config_path);
        print!("{}", config);
        return;
    }

    let root_addr = format!("{}:{}", config.root.ip, config.root.port);

    if is_server_running(&root_addr) {
        eprintln!("Sentinel is already running.");
        std::process::exit(1);
    }

    if no_daemon {
        println!("Sentinel server starting in foreground on {}...", root_addr);
        start_server(&root_addr);
        return;
    }

    println!("Sentinel server starting on {}...", root_addr);
    daemonize_and_start(&root_addr);
}

fn daemonize_and_start(addr: &str) {
    std::fs::create_dir_all("logs").ok();

    let stdout = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("logs/sentinel.log")
        .expect("Failed to open logs/sentinel.log");

    let stderr = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("logs/sentinel.log")
        .expect("Failed to open logs/sentinel.log");

    let daemon = Daemonize::new()
        .working_directory(".")
        .stdout(stdout)
        .stderr(stderr);

    match daemon.start() {
        Ok(_) => start_server(addr),
        Err(e) => {
            eprintln!("Failed to daemonize: {}", e);
            std::process::exit(1);
        }
    }
}

fn is_server_running(addr: &str) -> bool {
    TcpListener::bind(addr).is_err()
}

fn cli_send_command(cmd: &str) {
    let stream = match UnixStream::connect(SOCKET_PATH) {
        Ok(s) => s,
        Err(_) => {
            eprintln!("Sentinel server is not running.");
            std::process::exit(1);
        }
    };

    let mut writer = stream.try_clone().unwrap();
    let _ = writeln!(writer, "{}", cmd);
    let _ = writer.shutdown(Shutdown::Write);

    let reader = BufReader::new(&stream);
    for line in reader.lines().flatten() {
        println!("{}", line);
    }
}

fn print_help() {
    println!("Sentinel — Distributed Node Server\n");
    println!("USAGE:");
    println!("  sentinel [OPTIONS]\n");
    println!("OPTIONS:");
    println!("  -h, --help          Show this help message");
    println!("  -ls                 List all connected node IDs");
    println!("  --shutdown          Gracefully shut down the running server");
    println!("  --config-print      Print the loaded configuration");
    println!("  -c, --config PATH   Use a custom config file (default: config.toml)");
    println!("  --no-daemon         Run in foreground (for testing/debugging)\n");
    println!("EXAMPLES:");
    println!("  sentinel                Start the server (daemonized)");
    println!("  sentinel -ls            List connected nodes");
    println!("  sentinel --shutdown     Stop the running server");
    println!("  sentinel --config-print Show current config");
    println!("  sentinel -c test.toml   Start with a custom config");
}
