# Sentinel Bridge API Documentation

The Sentinel Bridge exposes an HTTP API for interacting with and managing connected clients over TCP.

**Base URL**: `http://<server_ip>:3737`

---

## 1. System Operations


### 1.1 `GET /status`
Check the operational status of the HTTP bridge server.

**Response**
```json
{
  "message": "Status: Active"
}
```

---

### 1.2 `POST /stop`
Triggers a shutdown signal across the root sentinel allowing graceful exit.

**Response**
```json
{
  "message": "Stopping Sentinel"
}
```

---

## 2. Client Management


### 2.1 `GET /clients`
Fetches a list of all currently connected clients and their initial metadata.

**Response**
```json
[
  {
    "id": 1,
    "name": "User-1",
    "register": "REG123"
  }
]
```

---

### 2.2 `POST /send`
Dispatches a raw text message down the TCP pipeline to a specific client ID.

**Payload**
```json
{
  "id": 1,
  "message": "Hello Client"
}
```

**Response**
- Success: `{"message": "Message sent"}`
- Error: `{"message": "Client not found"}`

---

## 3. Network Policy Management


### 3.1 `POST /policy`
Modifies the network `allowed_sites` list for a specific client or broadcasts to all clients (`*`). 

The provided `url` will automatically parsed and stripped down to its `domain`, then dispatched to the specified clients natively as `ACTION network ALLOW <domain>` or `ACTION network BLOCK <domain>`.

**Payload**
```json
{
  "id": "*",                // Client ID ("*"" for all connected clients)
  "action": "ALLOW",        // "ALLOW" or "BLOCK"
  "url": "https://example.com"
}
```

**Response**
- Success (Broadcast): `{"message": "Broadcasted policy to all clients"}`
- Success (Targeted): `{"message": "Policy sent"}`
- Error (Invalid ID): `{"message": "Client not found"}`
- Error (Invalid URL): `{"message": "Invalid URL"}`
- Error (Invalid Action): `{"message": "Invalid action (must be ALLOW or BLOCK)"}`

---

### 3.2 `GET /allowed_sites/:id`
Retrieves the currently tracked `allowed_sites` array for the requested client ID. By default, new clients connect with `lms.ssn.edu.in` whitelisted.

**URL Parameters**
- `:id` (Integer): The integer ID of the client.

**Response**
```json
{
  "id": 1,
  "allowed_sites": [
    "lms.ssn.edu.in",
    "example.com"
  ]
}
```

**Error Response**
```json
{
  "error": "Client not found"
}
```

---

## 4. WebSockets

### 4.1 `GET /kafka/ws`
Upgrades an incoming HTTP connection to a WebSocket for streaming Kafka log aggregates.
