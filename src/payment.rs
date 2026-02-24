//! Payment abstraction layer.
//!
//! Payment is **always validated server-side**. The client sends payment
//! details (card token, receipt, etc.) and the server decides whether the
//! payment succeeded. The client never controls the outcome.
//!
//! ## Development / testing
//! Set `DUMMY_PAYMENT_MODE=true` in `.env` to make every payment succeed
//! automatically so you can test billing flows without a real gateway.
//! This MUST be `false` (or absent) in production.
//!
//! ## Production
//! Implement [`PaymentProvider`] with your real gateway and register it as an
//! Axum [`Extension`](axum::Extension) in `main.rs`.

use serde_json::Value;
use std::future::Future;
use std::pin::Pin;

// ---------------------------------------------------------------------------
// Payment trait & associated types
// ---------------------------------------------------------------------------

/// Provider-agnostic description of a charge to make.
pub struct PaymentDetails {
    pub amount: f64,
    pub description: String,
    pub metadata: Option<Value>,
}

/// Result returned by a payment provider after attempting a charge.
pub struct PaymentOutcome {
    pub success: bool,
    /// Short identifier for the provider (e.g. `"stripe"`, `"dummy"`).
    pub provider: String,
    pub message: String,
    pub transaction_id: Option<String>,
}

/// Implement this trait for each payment backend and register it via
/// `app.layer(Extension(Arc::new(MyProvider) as Arc<dyn PaymentProvider>))`.
pub trait PaymentProvider: Send + Sync {
    fn charge<'a>(
        &'a self,
        details: &'a PaymentDetails,
    ) -> Pin<Box<dyn Future<Output = PaymentOutcome> + Send + 'a>>;
}

// ---------------------------------------------------------------------------
// Dummy provider — always succeeds. Use only in development/testing.
// Enabled automatically when DUMMY_PAYMENT_MODE=true.
// ---------------------------------------------------------------------------

pub struct DummyPaymentProvider;

impl PaymentProvider for DummyPaymentProvider {
    fn charge<'a>(
        &'a self,
        details: &'a PaymentDetails,
    ) -> Pin<Box<dyn Future<Output = PaymentOutcome> + Send + 'a>> {
        Box::pin(async move {
            PaymentOutcome {
                success: true,
                provider: "dummy".to_string(),
                message: format!("Dummy charge of ${:.2} approved.", details.amount),
                transaction_id: Some(format!("dummy_txn_{}", uuid::Uuid::new_v4())),
            }
        })
    }
}

