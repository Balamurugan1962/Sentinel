# Sentinel — Root Server

A TCP-based node management server that authenticates connecting nodes against a TOML config, tracks active connections, and exposes a CLI for runtime queries. Runs as a background daemon with per-node file logging.

## Install

```bash
chmod +x install.sh
./install.sh
source ~/.zshrc
```

This builds the release binary, copies it to `~/.local/bin/sentinel`, and adds it to your PATH.

## Architecture

```
┌──────────────────────────────────────────────────────┐
│                     main.rs                          │
│  CLI dispatcher — routes to server or IPC commands   │
│                                                      │
│  sentinel              → daemonize + start_server()  │
│  sentinel --no-daemon  → foreground + start_server() │
│  sentinel -ls          → IPC query via Unix socket   │
│  sentinel --shutdown   → IPC shutdown via Unix socket │
│  sentinel -h           → print help                  │
└───────┬──────────────────────────────────┬───────────┘
        │                                  │
        ▼                                  ▼
┌───────────────┐                ┌──────────────────┐
│  config.rs    │                │    server.rs     │
│               │                │                  │
│  Config       │◄──────────────►│  TCP listener    │
│  ├── Root     │  loaded once   │  ├── handshake   │
│  └── Node[]   │                │  ├── read_loop   │
│               │                │  └── logging     │
│  load_config()│                │                  │
└───────────────┘                │  CLI socket      │
                                 │  └── ls query    │
                                 └──────────────────┘
                                          │
                                          ▼
                                 ┌──────────────────┐
                                 │   logs/           │
                                 │  ├── sentinel.log │
                                 │  ├── 1.log        │
                                 │  ├── 2.log        │
                                 │  └── unknown.log  │
                                 └──────────────────┘
```

## Modules

### `config.rs` — Configuration

Deserializes `config.toml` into typed Rust structs.

| Struct   | Fields          | Purpose                      |
|----------|-----------------|------------------------------|
| `Config` | `root`, `nodes` | Top-level config container   |
| `Root`   | `ip`, `port`    | Address the server binds to  |
| `Node`   | `id`, `ip`      | Allowed node identity        |

### `server.rs` — Core Server

Runs two listeners in parallel:

**TCP Listener** — Accepts node connections on `root.ip:root.port`.

| Phase      | Function           | What it does                               |
|------------|--------------------|--------------------------------------------|
| Connect    | `handle_node`      | Validates peer IP against config           |
| Handshake  | `parse_handshake`  | Reads `HELLO <id>\n`, extracts node ID     |
| Register   | `register_node`    | Inserts into shared `ActiveNodes` map      |
| Data       | `read_loop`        | Reads messages, writes to `logs/<id>.log`  |
| Disconnect | `unregister_node`  | Removes from `ActiveNodes` on stream close |

**CLI Socket** — Unix domain socket at `/tmp/sentinel.sock` accepting `ls` queries.

### `main.rs` — Entry Point & CLI

| Flag            | Action                     |
|-----------------|----------------------------|
| *(none)*        | Daemonize and start server |
| `--no-daemon`   | Start server in foreground |
| `-c PATH`       | Use custom config file     |
| `-ls`           | Query connected nodes      |
| `--shutdown`    | Stop the running server    |
| `-h`, `--help`  | Print usage information    |

## Connection Protocol

```
Client                          Server
  │──── TCP connect ──────────────►│  verify peer IP ∈ config
  │──── "HELLO 1\n" ─────────────►│  parse node ID, verify in config
  │◄─── (registered) ─────────────│  logged to logs/1.log
  │──── "any message\n" ─────────►│  logged to logs/1.log
  │──── disconnect ───────────────►│  removed from active nodes
```

## File Structure

```
root/
├── Cargo.toml
├── config.toml
├── install.sh
├── src/
│   ├── main.rs
│   ├── config.rs
│   └── server.rs
├── logs/                  # created at runtime
└── tests/
    ├── test_config.toml
    ├── test_clients.rs
    ├── test1/client.c
    └── test2/client.c
```

## Quick Start

```bash
# install globally
./install.sh && source ~/.zshrc

# start server
sentinel

# list connected nodes
sentinel -ls

# connect a test client
cd tests/test2 && gcc -o client client.c
./client 1

# stop server
sentinel --shutdown
```
