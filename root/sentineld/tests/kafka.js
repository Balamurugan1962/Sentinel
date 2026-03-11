const WebSocket = require("ws");

const ws = new WebSocket("ws://127.0.0.1:3737/kafka/ws");

ws.on("open", () => {
  console.log("Connected");

  ws.send(
    JSON.stringify({
      topic: "27-browser",
    }),
  );
});

ws.on("message", (data) => {
  console.log("Kafka message:", data.toString());
});

ws.on("close", () => {
  console.log("Connection closed");
});

ws.on("error", (err) => {
  console.error("Error:", err);
});
