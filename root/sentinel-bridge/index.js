#!/usr/bin/env node

/**
 * Sentinel Bridge Server
 * Run this on the client machine: node sentinel-bridge.js
 * It bridges HTTP requests from the browser to the local Unix socket.
 */

const net = require("net");
const http = require("http");

const SOCKET_PATH = "/tmp/sentinel.sock";
const BRIDGE_PORT = 7373;

function sendToSentinel(command) {
  return new Promise((resolve, reject) => {
    const client = net.createConnection(SOCKET_PATH, () => {
      client.write(command);
    });

    let data = "";
    client.on("data", (chunk) => {
      data += chunk.toString();
    });
    client.on("end", () => resolve(data));
    client.on("error", (err) => reject(err));

    // Timeout after 5 seconds
    setTimeout(() => {
      client.destroy();
      reject(new Error("Connection timed out"));
    }, 5000);
  });
}

const server = http.createServer(async (req, res) => {
  // Allow browser to connect (CORS)
  res.setHeader("Access-Control-Allow-Origin", "*");
  res.setHeader("Access-Control-Allow-Methods", "POST, OPTIONS");
  res.setHeader("Access-Control-Allow-Headers", "Content-Type");
  res.setHeader("Content-Type", "application/json");

  // Handle preflight
  if (req.method === "OPTIONS") {
    res.writeHead(200);
    res.end();
    return;
  }

  if (req.method !== "POST") {
    res.writeHead(404);
    res.end(JSON.stringify({ error: "Not found" }));
    return;
  }

  let body = "";
  req.on("data", (chunk) => {
    body += chunk;
  });
  req.on("end", async () => {
    try {
      const { command } = JSON.parse(body || "{}");
      const result = await sendToSentinel(command || "-ls");
      res.writeHead(200);
      res.end(JSON.stringify({ success: true, data: result }));
    } catch (err) {
      res.writeHead(503);
      res.end(JSON.stringify({ success: false, error: err.message }));
    }
  });
});

server.listen(BRIDGE_PORT, "127.0.0.1", () => {
  console.log(`✅ Sentinel bridge running on http://localhost:${BRIDGE_PORT}`);
  console.log(`   Bridging to: ${SOCKET_PATH}`);
  console.log(`   Press Ctrl+C to stop.`);
});
