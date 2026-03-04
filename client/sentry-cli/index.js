#!/usr/bin/env node

const net = require("net");

const SOCKET_PATH = "/tmp/sentry.sock";

const command = process.argv[2];

if (!command) {
  console.log(`
Sentinel CLI

Usage:
  sentry-cli status
  sentry-cli stop
`);
  process.exit(1);
}

let cmd;

switch (command) {
  case "status":
    cmd = "-status";
    break;

  case "stop":
    cmd = "-stop";
    break;

  default:
    console.log("Unknown command");
    process.exit(1);
}

const client = net.createConnection(SOCKET_PATH);

client.on("connect", () => {
  client.write(cmd);
});

client.on("data", (data) => {
  process.stdout.write(data.toString());
  client.end();
});

client.on("error", (err) => {
  console.error("Failed to connect to sentry daemon");
  console.error(err.message);
  process.exit(1);
});
