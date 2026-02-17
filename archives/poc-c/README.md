# Sentinel Client – C POC

This is a proof-of-concept client agent written in C.

## Responsibilities
- Load configuration from config.toml
- Connect to root server
- Send periodic heartbeat
- Log system state (non-invasive)

## Non-Goals
- No keystroke logging
- No screen capture
- No packet sniffing
- No kernel-level hooks

## Build
make

## Run
sentinel-client.exe

## Author
**Rahul V S**