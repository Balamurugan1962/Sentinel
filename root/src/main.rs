use std::io::{BufRead, BufReader, Write};
use std::net::{Ipv4Addr, Shutdown, TcpListener};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::{env, fs};

use daemonize::Daemonize;

use crate::config::load_config;
use crate::io::prompt_input;
use crate::server::{SOCKET_PATH, start_server};

mod config;
mod io;
mod server;

fn get_base_dir() -> PathBuf {
    return dirs::home_dir().unwrap().join(".sentinel");
}
fn main() {
    let base_dir = get_base_dir();
    std::fs::create_dir_all(&base_dir).unwrap();

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

    let mut config_path = base_dir.join("config.toml");
    let mut no_daemon = false;
    let mut print_config = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-c" | "--config" => {
                i += 1;
                if i < args.len() {
                    config_path = PathBuf::from(&args[i]);
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

    if !Path::new(&config_path).exists() {
        println!("Config file not found at '{}'", config_path.display());
        println!("Let's create one.");

        // Ask user for IP and port
        let ip_str = prompt_input("Enter root IP (e.g., 192.168.1.10): ");
        let port_str = prompt_input("Enter root port (e.g., 8080): ");

        // Validate IP
        let ip: Ipv4Addr = match ip_str.parse() {
            Ok(ip) => ip,
            Err(_) => {
                eprintln!("Invalid IP address '{}'", ip_str);
                std::process::exit(1);
            }
        };

        // Validate port
        let port: u16 = match port_str.parse() {
            Ok(port) => port,
            Err(_) => {
                eprintln!("Invalid port '{}'", port_str);
                std::process::exit(1);
            }
        };

        // Save a minimal config.toml
        let toml_content = format!("[root]\nip = \"{}\"\nport = {}", ip, port);

        fs::write(&config_path, toml_content).expect("Failed to write config.toml");
        println!("Config saved to '{}'", config_path.display());
    }

    if fs::metadata(&config_path).is_err() {
        eprintln!(
            "Error: Cannot access config file '{}'",
            config_path.display()
        );
        std::process::exit(1);
    }

    let config = load_config(&config_path);

    if print_config {
        println!(
            "Sentinel — Loaded config from '{}'\n",
            config_path.display()
        );
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
    daemonize_and_start(&root_addr, &base_dir);
}

fn daemonize_and_start(addr: &str, base_dir: &Path) {
    let log_dir = base_dir.join("logs");
    let sentinel_log = log_dir.join("sentinel.log");
    let sentinel_err = log_dir.join("errors.log");

    if log_dir.exists() {
        std::fs::remove_dir_all(&log_dir).ok();
    }

    std::fs::create_dir_all(&log_dir).expect("Failed to create logs directory");

    let stdout = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&sentinel_log)
        .expect("Failed to open logs/sentinel.log");

    let stderr = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&sentinel_err)
        .expect("Failed to open logs/sentinel.log");

    let daemon = Daemonize::new()
        .working_directory(&base_dir)
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
