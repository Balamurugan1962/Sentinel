#!/usr/bin/env node

const net = require("net");

const SOCKET_PATH = "/tmp/sentinel.sock";

function formatTable(data) {
  if (!data.length) {
    console.log("No clients connected");
    return;
  }

  const header = ["Client", "Name", "Reg"];

  const widths = [10, 20, 20];

  const line = (cols) =>
    cols.map((c, i) => String(c).padEnd(widths[i])).join(" | ");

  console.log(line(header));
  console.log("-".repeat(10) + "-+-" + "-".repeat(20) + "-+-" + "-".repeat(20));

  for (const c of data) {
    console.log(line([c.id, c.name, c.reg]));
  }
}

function sendCommand(command) {
  const client = net.createConnection(SOCKET_PATH);

  let buffer = "";

  client.on("connect", () => {
    client.write(command);
  });

  client.on("data", (data) => {
    buffer += data.toString();
  });

  client.on("end", () => {
    try {
      const json = JSON.parse(buffer);

      if (Array.isArray(json)) {
        formatTable(json);
      } else {
        console.log(json);
      }
    } catch {
      console.log(buffer);
    }
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
