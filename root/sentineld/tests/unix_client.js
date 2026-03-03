const net = require("net");

const SOCKET_PATH = "/tmp/sentinel.sock";

// Get CLI arguments (skip node + file)
const command = process.argv.slice(2).join(" ");

if (!command) {
  console.log("Usage:");
  console.log("  node unix_client.js -ls");
  console.log("  node unix_client.js -send <id> <message>");
  process.exit(0);
}

// Create Unix socket connection
const client = net.createConnection(SOCKET_PATH, () => {
  // Send command when connected
  client.write(command);
});

// Handle response from daemon
client.on("data", (data) => {
  console.log(data.toString());
  client.end(); // close after response
});

// Handle errors
client.on("error", (err) => {
  console.error("Connection error:", err.message);
  process.exit(1);
});
