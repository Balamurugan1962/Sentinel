#!/usr/bin/env node

const BASE_URL = "http://127.0.0.1:3737";

function formatTable(data) {
  if (!data.length) {
    console.log("No clients connected");
    return;
  }

  const header = ["Client", "Name", "Register"];
  const widths = [10, 20, 20];

  const line = (cols) =>
    cols.map((c, i) => String(c).padEnd(widths[i])).join(" | ");

  console.log(line(header));
  console.log("-".repeat(10) + "-+-" + "-".repeat(20) + "-+-" + "-".repeat(20));

  for (const c of data) {
    console.log(line([c.id, c.name, c.register]));
  }
}

async function get(path) {
  try {
    const res = await fetch(`${BASE_URL}${path}`);
    const data = await res.json();
    return data;
  } catch (err) {
    console.error("Cannot connect to sentinel server");
    console.error(err.message);
    process.exit(1);
  }
}

async function post(path, body = {}) {
  try {
    const res = await fetch(`${BASE_URL}${path}`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(body),
    });

    const data = await res.json();
    return data;
  } catch (err) {
    console.error("Cannot connect to sentinel server");
    console.error(err.message);
    process.exit(1);
  }
}

async function main() {
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
    case "status": {
      const res = await get("/status");
      console.log(res.message || res);
      break;
    }

    case "ls": {
      const res = await get("/clients");

      if (Array.isArray(res)) {
        formatTable(res);
      } else {
        console.log(res);
      }

      break;
    }

    case "stop": {
      const res = await post("/stop");
      console.log(res.message || res);
      break;
    }

    case "send": {
      if (args.length < 3) {
        console.log("Usage: sentinel send <id> <message>");
        process.exit(1);
      }

      const id = parseInt(args[1]);
      const message = args.slice(2).join(" ");

      const res = await post("/send", { id, message });

      console.log(res.message || res);
      break;
    }

    default:
      console.log("Unknown command");
  }
}

main();
