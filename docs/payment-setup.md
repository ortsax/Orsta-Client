# Payment Provider Setup

Orsta-Client delegates payment processing to a `PaymentProvider` implementation that you supply. This keeps the billing logic decoupled from any specific gateway (Stripe, PayPal, your own backend, etc.).

---

## How it works

1. The client calls `POST /billing/enable-api-key` with an `amount`, optional `description`, and optional `metadata`.
2. The server calls `PaymentProvider::charge()` with those details.
3. If `PaymentOutcome::success` is `true`, the API key is activated and billing records are updated.
4. If `success` is `false`, a `402 Payment Required` is returned to the client — the API key is **not** activated.

The client **never** controls the outcome. Only your server-side implementation decides whether a charge succeeded.

---

## Development mode (no real payments)

Set `DUMMY_PAYMENT_MODE=true` in your `.env` file to make every charge auto-succeed without hitting any gateway. **Never enable this in production.**

```env
DUMMY_PAYMENT_MODE=true
```

> This requires you to still register a `PaymentProvider` (see below). Use the included `DummyPaymentProvider` for local development.

---

## Implementing a PaymentProvider

### 1. Define your provider in `src/payment.rs`

```rust
pub struct StripeProvider {
    pub secret_key: String,
}

impl PaymentProvider for StripeProvider {
    fn charge<'a>(
        &'a self,
        details: &'a PaymentDetails,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = PaymentOutcome> + Send + 'a>> {
        Box::pin(async move {
            // Call Stripe (or any gateway) here.
            // details.amount      — amount to charge
            // details.description — charge description
            // details.metadata    — arbitrary JSON from client (card token, etc.)

            // Example (pseudo-code):
            // let result = stripe::charge(&self.secret_key, details.amount, &details.metadata).await;
            // PaymentOutcome {
            //     success: result.status == "succeeded",
            //     provider: "stripe".to_string(),
            //     message: result.message,
            //     transaction_id: Some(result.id),
            // }

            // Placeholder — replace with real gateway call:
            PaymentOutcome {
                success: false,
                provider: "stripe".to_string(),
                message: "Not implemented".to_string(),
                transaction_id: None,
            }
        })
    }
}
```

### 2. Register it in `src/main.rs`

```rust
use std::sync::Arc;
use crate::payment::{PaymentProvider, StripeProvider};
use axum::Extension;

let payment_provider: Arc<dyn PaymentProvider> = Arc::new(StripeProvider {
    secret_key: std::env::var("STRIPE_SECRET_KEY").expect("STRIPE_SECRET_KEY must be set"),
});

let app = route::start_client_api_service(Arc::clone(&orchestrator))
    .layer(Extension(payment_provider));
```

---

## Client request reference

### `POST /billing/enable-api-key`

| Field | Type | Required | Description |
|---|---|---|---|
| `amount` | `f64` | ✅ | Amount to charge |
| `description` | `string` | ❌ | Human-readable charge reason |
| `metadata` | `object` | ❌ | Provider-specific data (card token, payment-intent ID, receipt, etc.) |

**Example:**
```json
{
  "amount": 9.99,
  "description": "Monthly API access",
  "metadata": { "payment_intent_id": "pi_abc123" }
}
```

**Success response (`200`):**
```json
{
  "ok": true,
  "message": "API key activated",
  "transaction_id": "txn_abc123",
  "provider": "stripe",
  "amount_charged": 9.99
}
```

**Payment failed (`402`):**
```json
{
  "error": "Payment failed",
  "reason": "Card declined",
  "provider": "stripe"
}
```

### `POST /billing/disable-api-key`

No body required. Deactivates the authenticated user's API key.

### `GET /billing/api-key-status`

Returns the user's API key and its active state.

```json
{ "api_key": "a3f9...c1d2", "active": true }
```

### `GET /billing/summary`

Returns billing totals for the authenticated user.

```json
{
  "amount_in_wallet": 50.00,
  "amount_spent": 9.99,
  "total_amount_spent": 19.98,
  "average_hourly_consumption": 0.014
}
```

---

## `PaymentOutcome` fields

| Field | Type | Description |
|---|---|---|
| `success` | `bool` | Whether the charge succeeded |
| `provider` | `String` | Short name of the gateway (`"stripe"`, `"paypal"`, etc.) |
| `message` | `String` | Human-readable result message |
| `transaction_id` | `Option<String>` | Gateway transaction ID (returned to client on success) |
