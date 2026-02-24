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

## 7. Billing

All billing endpoints require a JWT via `Authorization: Bearer` or the `orsta_session` cookie.

### Enable API Key (pay & activate)

```bash
curl -X POST http://localhost:3000/billing/enable-api-key \
  -H "Authorization: Bearer <your_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "amount": 9.99,
    "description": "Monthly API access",
    "metadata": { "plan": "starter" }
  }'
```

**Response (success):**
```json
{
  "ok": true,
  "message": "API key activated",
  "transaction_id": "txn_abc123",
  "provider": "stripe",
  "amount_charged": 9.99
}
```

**Response (payment failed — 402):**
```json
{
  "error": "Payment failed",
  "reason": "Card declined",
  "provider": "stripe"
}
```

### Disable API Key

```bash
curl -X POST http://localhost:3000/billing/disable-api-key \
  -H "Authorization: Bearer <your_token>"
```

**Response:**
```json
{ "ok": true, "message": "API key deactivated" }
```

### API Key Status

```bash
curl http://localhost:3000/billing/api-key-status \
  -H "Authorization: Bearer <your_token>"
```

**Response:**
```json
{ "api_key": "a3f9...c1d2", "active": true }
```

### Billing Summary

```bash
curl http://localhost:3000/billing/summary \
  -H "Authorization: Bearer <your_token>"
```

**Response:**
```json
{
  "amount_in_wallet": 50.00,
  "amount_spent": 9.99,
  "total_amount_spent": 19.98,
  "average_hourly_consumption": 0.014
}
```


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
