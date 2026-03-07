#!/usr/bin/env node

/**
 * Sentinel Bridge Server
 * ----------------------
 * Bridges HTTP requests from the Next.js frontend to the Sentinel
 * daemon which listens on a Unix domain socket at /tmp/sentry.sock.
 *
 * Run this alongside `npm run dev`:
 *   node src/lib/sentinel-bridge.js
 *
 * Endpoints:
 *   POST http://localhost:7373
 *   Body: { "command": "..." }
 *
 * Commands forwarded to Sentinel:
 *   -status         → Check if Sentinel is running
 *   -stop           → Stop the Sentinel daemon
 *   info --name <n> --reg <r>  → Register student with the daemon
 */

const http = require("http");
const net = require("net");

const BRIDGE_PORT = 7373;
const SOCKET_PATH = "/tmp/sentry.sock";

/**
 * Send a command to the Sentinel Unix domain socket and
 * return the response as a string.
 */
function sendToSentinel(command) {
  return new Promise((resolve, reject) => {
    const client = net.createConnection(SOCKET_PATH);

    let response = "";
    let settled = false;

    const finish = (val) => {
      if (settled) return;
      settled = true;
      resolve(val);
    };

    client.on("connect", () => {
      client.write(command);
    });

    client.on("data", (data) => {
      response += data.toString();
    });

    // Daemon closed the connection normally (also happens after -stop)
    client.on("end", () => finish(response || "ok"));
    client.on("close", () => finish(response || "ok"));

    client.on("error", (err) => {
      // If we already got some data, treat as success
      if (response) {
        finish(response);
        return;
      }
      if (settled) return;
      settled = true;
      reject(err);
    });

    // Safety timeout – if Sentinel doesn't respond within 5s, bail out
    client.setTimeout(5000, () => {
      client.destroy();
      finish(response || "timeout");
    });
  });
}

const server = http.createServer(async (req, res) => {
  // Enable CORS so the browser (Next.js dev server on :3000) can call us
  res.setHeader("Access-Control-Allow-Origin", "*");
  res.setHeader("Access-Control-Allow-Methods", "POST, OPTIONS");
  res.setHeader("Access-Control-Allow-Headers", "Content-Type");

  // Handle CORS preflight
  if (req.method === "OPTIONS") {
    res.writeHead(204);
    res.end();
    return;
  }

  if (req.method !== "POST") {
    res.writeHead(405, { "Content-Type": "application/json" });
    res.end(JSON.stringify({ error: "Method Not Allowed" }));
    return;
  }

  // Parse request body
  let body = "";
  req.on("data", (chunk) => (body += chunk));
  req.on("end", async () => {
    let command;
    try {
      const parsed = JSON.parse(body);
      command = parsed.command;
    } catch {
      res.writeHead(400, { "Content-Type": "application/json" });
      res.end(JSON.stringify({ error: "Invalid JSON body" }));
      return;
    }

    if (!command) {
      res.writeHead(400, { "Content-Type": "application/json" });
      res.end(JSON.stringify({ error: "Missing 'command' field" }));
      return;
    }

    console.log(`[Sentinel Bridge] Forwarding command: ${command}`);

    try {
      const data = await sendToSentinel(command);
      res.writeHead(200, { "Content-Type": "application/json" });
      res.end(JSON.stringify({ data }));
    } catch (err) {
      console.error(`[Sentinel Bridge] Error: ${err.message}`);
      res.writeHead(503, { "Content-Type": "application/json" });
      res.end(JSON.stringify({ error: "Sentinel daemon is not running" }));
    }
  });
});

server.listen(BRIDGE_PORT, "127.0.0.1", () => {
  console.log(`[Sentinel Bridge] Listening on http://127.0.0.1:${BRIDGE_PORT}`);
  console.log(`[Sentinel Bridge] Forwarding to Unix socket: ${SOCKET_PATH}`);
});
