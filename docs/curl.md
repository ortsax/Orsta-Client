# cURL — Orsta-Client Usage

---

## 1. Sign Up

```bash
curl -X POST http://localhost:3000/auth/signup \
  -H "Content-Type: application/json" \
  -d '{
    "username": "alice",
    "email": "alice@example.com",
    "password": "supersecret",
    "passkey": null
  }'
```

**Response:**
```json
{
  "token": "<jwt>",
  "user_id": 1,
  "username": "alice",
  "eakey": "a3f9...c1d2"
}
```

---

## 2. Log In

```bash
curl -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "alice@example.com",
    "password": "supersecret"
  }'
```

Save the token from the response for use below.

---

## 3. Log Out

```bash
curl -X POST http://localhost:3000/auth/logout \
  -H "Cookie: orsta_session=<your_token>"
```

---

## 4. Check Current User (`/me`)

```bash
curl http://localhost:3000/me \
  -H "Authorization: Bearer <your_token>"
```

**Response:**
```json
{
  "user_id": "1",
  "username": "alice"
}
```

---

## 5. WebSocket Connection

cURL supports WebSocket upgrades since version **7.86.0**.

```bash
# Connect and send a ping message
curl --http1.1 \
  -H "Authorization: Bearer <your_token>" \
  --no-buffer \
  "ws://localhost:3000/ws"
```

For interactive WebSocket testing from the command line, use [websocat](https://github.com/vi/websocat):

```bash
# Install websocat (Windows via scoop)
scoop install websocat

# Connect with token in header
websocat -H "Authorization: Bearer <your_token>" ws://localhost:3000/ws
```

Once connected, type JSON messages and press Enter:

```
{"action":"ping"}
{"action":"whoami"}
{"action":"ping","payload":{"note":"hello"}}
```

---

## 6. Storing the Cookie with cURL

Sign in and persist the session cookie to a file for subsequent requests:

```bash
# Login and save cookie
curl -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"alice@example.com","password":"supersecret"}' \
  -c cookies.txt

# Use saved cookie
curl http://localhost:3000/me -b cookies.txt
```
