# Orsta

## Description

The purpose of this client is to provide a unified backend interface for the administration and orchestration of Orsta WhatsApp instances. It serves as the primary management layer for handling customer accounts, allowing for seamless provisioning, monitoring, and real-time configuration of WA service environments.

## Usage

All communication with Orsta-Client happens over two channels:

- **HTTP** — authentication (`/auth/signup`, `/auth/login`, `/auth/logout`) and billing (`/billing/*`)
- **WebSocket** — real-time instance control once a session is established

### 1. Authentication

**Sign up**

```http
POST /auth/signup
Content-Type: application/json

{
  "username": "alice",
  "email": "alice@example.com",
  "password": "supersecret",
  "passkey": null
}
```

**Log in**

```http
POST /auth/login
Content-Type: application/json

{
  "email": "alice@example.com",
  "password": "supersecret"
}
```

Both responses return a JSON body with `token`, `user_id`, `username`, and `eakey`, and also set an `orsta_session` cookie (HttpOnly, 7-day expiry).

### 2. WebSocket Connection

Connect to `ws://<host>:<port>/ws` and pass your token via **one** of:

| Method | How                                                    |
| ------ | ------------------------------------------------------ |
| Cookie | Browser sends `orsta_session` cookie automatically     |
| Header | `Authorization: Bearer <token>` on the upgrade request |

Once connected the server sends a `connected` confirmation frame:

```json
{ "action": "connected", "data": { "user_id": "1", "username": "alice" } }
```

### 3. Sending Messages

All WebSocket messages are JSON envelopes:

```json
{ "action": "<action>", "payload": {} }
```

| Action   | Description               | Example response                                                          |
| -------- | ------------------------- | ------------------------------------------------------------------------- |
| `ping`   | Heartbeat check           | `{ "action": "pong" }`                                                    |
| `whoami` | Returns current user info | `{ "action": "whoami", "data": { "user_id": "1", "username": "alice" } }` |

See [`docs/`](./docs) for full usage examples in TypeScript, Go, Python, and cURL.

### 4. Billing

Billing endpoints require a valid `Authorization: Bearer <token>` header.

**Activate API key** (charges the user)

```http
POST /billing/enable-api-key
Authorization: Bearer <token>
Content-Type: application/json

{
  "amount": 9.99,
  "description": "Monthly API access",
  "metadata": { "plan": "starter" }
}
```

Response:
```json
{
  "ok": true,
  "message": "API key activated",
  "transaction_id": "txn_abc123",
  "provider": "stripe",
  "amount_charged": 9.99
}
```

**Deactivate API key**

```http
POST /billing/disable-api-key
Authorization: Bearer <token>
```

**Check API key status**

```http
GET /billing/api-key-status
Authorization: Bearer <token>
```

**Billing summary**

```http
GET /billing/summary
Authorization: Bearer <token>
```

> Payment processing requires a [`PaymentProvider`](./docs/payment-setup.md) implementation. See [`docs/payment-setup.md`](./docs/payment-setup.md) for setup instructions.

## License

This project is licensed under the [MIT License](./LICENSE.md).
