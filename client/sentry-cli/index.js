#!/usr/bin/env node

const BASE = "http://127.0.0.1:7373";

const args = process.argv.slice(2);

if (args.length === 0) {
  console.log(`
Sentinel CLI

Usage:
  sentry-cli status
  sentry-cli stop
  sentry-cli logout
  sentry-cli info --name <name>
  sentry-cli info --reg <reg>
  sentry-cli info --name <name> --reg <reg>
`);
  process.exit(1);
}

async function request(path, method = "GET", body = null) {
  try {
    const res = await fetch(`${BASE}${path}`, {
      method,
      headers: {
        "Content-Type": "application/json",
      },
      body: body ? JSON.stringify(body) : undefined,
    });

    const text = await res.text();
    console.log(text);
  } catch (err) {
    console.error("Failed to connect to sentry daemon");
    console.error(err.message);
    process.exit(1);
  }
}

async function main() {
  switch (args[0]) {
    case "status":
      await request("/status");
      break;

    case "stop":
      await request("/stop", "POST");
      break;

    case "logout":
      await request("/logout", "POST");
      break;

    case "info": {
      let name = null;
      let reg = null;

      for (let i = 1; i < args.length; i++) {
        if (args[i] === "--name" && args[i + 1]) {
          name = args[i + 1];
          i++;
        } else if (args[i] === "--reg" && args[i + 1]) {
          reg = args[i + 1];
          i++;
        }
      }

      if (!name && !reg) {
        console.log(`
Usage:
  sentry-cli info --name <name>
  sentry-cli info --reg <reg>
  sentry-cli info --name <name> --reg <reg>
`);
        process.exit(1);
      }

      await request("/info", "POST", { name, reg });
      break;
    }

    default:
      console.log("Unknown command");
      process.exit(1);
  }
}

main();
