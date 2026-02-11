use crate::{config::load_config, server::start_server};

mod config;
mod server;

fn main() {
    let config = load_config("config.toml");

    let root_addr = format!("{}:{}", config.root.ip, config.root.port);

    std::thread::spawn(move || {
        start_server(&root_addr);
    });

    loop {}
}
