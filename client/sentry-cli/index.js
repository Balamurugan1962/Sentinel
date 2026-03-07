#!/usr/bin/env node

const net = require("net");

const SOCKET_PATH = "/tmp/sentry.sock";

const args = process.argv.slice(2);

if (args.length === 0) {
  console.log(`
Sentinel CLI

Usage:
  sentry-cli status
  sentry-cli stop
  sentry-cli info --name <name>
  sentry-cli info --reg <reg>
`);
  process.exit(1);
}

let cmd;

switch (args[0]) {
  case "status":
    cmd = "-status";
    break;

  case "stop":
    cmd = "-stop";
    break;

  case "info":
    if (args[1] === "--name" && args[2] && args[3] === "--reg") {
      cmd = `info --name ${args[2]} --reg ${args[4]}`;
    } else if (args[1] === "--reg" && args[2] && args[3] === "--name") {
      cmd = `info --reg ${args[2]} --name ${args[4]}`;
    } else if (args[1] === "--name" && args[2]) {
      cmd = `info --name ${args[2]}`;
    } else if (args[1] === "--reg" && args[2]) {
      cmd = `info --reg ${args[2]}`;
    } else {
      console.log(`
Usage:
  sentry-cli info --name <name>
  sentry-cli info --reg <reg>
`);
      process.exit(1);
    }
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
