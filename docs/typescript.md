# TypeScript — Orsta-Client Usage

Install dependencies:
```bash
npm install ws
npm install -D @types/ws
```

---

## 1. Sign Up

```typescript
const res = await fetch("http://localhost:3000/auth/signup", {
  method: "POST",
  headers: { "Content-Type": "application/json" },
  body: JSON.stringify({
    username: "alice",
    email: "alice@example.com",
    password: "supersecret",
    passkey: null,
  }),
});

const data = await res.json();
// data.token  — JWT for WebSocket auth
// data.eakey  — your unique encrypted access key
console.log(data);
```

---

## 2. Log In

```typescript
const res = await fetch("http://localhost:3000/auth/login", {
  method: "POST",
  headers: { "Content-Type": "application/json" },
  body: JSON.stringify({
    email: "alice@example.com",
    password: "supersecret",
  }),
});

const { token } = await res.json();
```

---

## 3. WebSocket Connection (Node.js)

```typescript
import WebSocket from "ws";

const token = "<your_jwt_token>";

const ws = new WebSocket("ws://localhost:3000/ws", {
  headers: {
    Authorization: `Bearer ${token}`,
  },
});

ws.on("open", () => {
  console.log("Connected to Orsta-Client");

  // Ping
  ws.send(JSON.stringify({ action: "ping" }));
});

ws.on("message", (data: Buffer) => {
  const msg = JSON.parse(data.toString());
  console.log("Received:", msg);
});

ws.on("close", () => console.log("Disconnected"));
ws.on("error", (err) => console.error("Error:", err));
```

---

## 4. WebSocket Connection (Browser)

> The browser sends the `orsta_session` cookie automatically if you logged in via `fetch` with `credentials: "include"`.

```typescript
const ws = new WebSocket("ws://localhost:3000/ws");

ws.addEventListener("open", () => {
  console.log("Connected");
  ws.send(JSON.stringify({ action: "whoami" }));
});

ws.addEventListener("message", (event) => {
  const msg = JSON.parse(event.data);
  console.log("Received:", msg);
});
```

---

## 5. Sending Messages

```typescript
// Ping
ws.send(JSON.stringify({ action: "ping" }));

// Who am I?
ws.send(JSON.stringify({ action: "whoami" }));

// Custom action with payload
ws.send(JSON.stringify({ action: "some_action", payload: { key: "value" } }));
```

---

## 6. Log Out

```typescript
await fetch("http://localhost:3000/auth/logout", { method: "POST" });
```
