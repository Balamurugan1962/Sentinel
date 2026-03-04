#!/usr/bin/env node

const net = require("net");

const SOCKET_PATH = "/tmp/sentinel.sock";

function sendCommand(command) {
  const client = net.createConnection(SOCKET_PATH);

  client.on("connect", () => {
    client.write(command);
  });

  client.on("data", (data) => {
    process.stdout.write(data.toString());
    client.end();
  });

  client.on("error", (err) => {
    console.error("Cannot connect to sentinel daemon");
    console.error(err.message);
    process.exit(1);
  });
}

const args = process.argv.slice(2);

if (args.length === 0) {
  console.log(`
Sentinel CLI

Usage:
  sentinel status
  sentinel ls
  sentinel stop
  sentinel send <id> <message>
`);
  process.exit(0);
}

const cmd = args[0];

switch (cmd) {
  case "status":
    sendCommand("-status");
    break;

  case "ls":
    sendCommand("-ls");
    break;

  case "stop":
    sendCommand("-stop");
    break;

  case "send":
    if (args.length < 3) {
      console.log("Usage: sentinel send <id> <message>");
      process.exit(1);
    }

    const id = args[1];
    const message = args.slice(2).join(" ");

    sendCommand(`-send ${id} ${message}`);
    break;

  default:
    console.log("Unknown command");
}
