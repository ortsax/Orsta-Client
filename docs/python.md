# Python — Orsta-Client Usage

Install dependencies:
```bash
pip install requests websockets
```

---

## 1. Sign Up

```python
import requests

resp = requests.post("http://localhost:3000/auth/signup", json={
    "username": "alice",
    "email": "alice@example.com",
    "password": "supersecret",
    "passkey": None,
})

data = resp.json()
token = data["token"]
eakey = data["eakey"]

print(f"Token: {token}")
print(f"EAKey: {eakey}")
```

---

## 2. Log In

```python
import requests

resp = requests.post("http://localhost:3000/auth/login", json={
    "email": "alice@example.com",
    "password": "supersecret",
})

token = resp.json()["token"]
```

---

## 3. WebSocket Connection

```python
import asyncio
import json
import websockets

async def connect(token: str):
    uri = "ws://localhost:3000/ws"
    headers = {"Authorization": f"Bearer {token}"}

    async with websockets.connect(uri, additional_headers=headers) as ws:
        # Receive welcome frame
        welcome = json.loads(await ws.recv())
        print("Connected:", welcome)

        # Ping
        await ws.send(json.dumps({"action": "ping"}))
        print("Pong:", json.loads(await ws.recv()))

        # Who am I?
        await ws.send(json.dumps({"action": "whoami"}))
        print("Whoami:", json.loads(await ws.recv()))

asyncio.run(connect(token))
```

---

## 4. Sending Messages

```python
async def send_message(ws, action: str, payload: dict = None):
    msg = {"action": action}
    if payload:
        msg["payload"] = payload
    await ws.send(json.dumps(msg))
    response = json.loads(await ws.recv())
    return response

# Inside an async context:
# response = await send_message(ws, "ping")
# response = await send_message(ws, "whoami")
```

---

## 5. Log Out

```python
requests.post("http://localhost:3000/auth/logout")
```

---

## 6. Full Example

```python
import asyncio
import json
import requests
import websockets

BASE = "http://localhost:3000"
WS   = "ws://localhost:3000/ws"

def login(email: str, password: str) -> str:
    r = requests.post(f"{BASE}/auth/login", json={"email": email, "password": password})
    r.raise_for_status()
    return r.json()["token"]

async def main():
    token = login("alice@example.com", "supersecret")

    async with websockets.connect(WS, additional_headers={"Authorization": f"Bearer {token}"}) as ws:
        print(json.loads(await ws.recv()))  # welcome

        await ws.send(json.dumps({"action": "ping"}))
        print(json.loads(await ws.recv()))  # pong

asyncio.run(main())
```
