# Centralized Monitoring System

CMS is a powerful and flexible tool for monitoring and controlling distributed systems. It provides a secure lab monitoring daemon.

# Sentinel:

Root server is known as Sentinel, and it creates multiple pooled threads, and creates a duplex based TCP connection

## Sentinel is part of:

### Sentineld:

Core daemon running in the background.

### Sentinel-cli:

Nodejs based cli, that connects sentineld with Unix Sockets.

# Sentry:

Client is know as sentry, and it connects to sentinel via TCP, has polling and duplex based communication, with that it also exposed Unix socket for internel communication.

## Sentry is part of:

### Sentryd (for now it is sentry):

Core daemon running in the background.

### Sentry-cli:

Nodejs based cli, that connects sentry with Unix Sockets.



# Installation:
for devs

clone the repo:
```bash
git clone https://github.com/Balamurugan1962/Sentinel
```

# for root system:
```bash
cd Sentinel/root/sentineld
cargo run
```
to check the status:
```bash
cd Sentinel/root/sentinel-cli
node index.js -status
```

## For client system:

```bash
cd Sentinel/client/sentry
cargo run
```
to check the status:
```bash
cd Sentinel/client/sentry-cli
node index.js -status
```
