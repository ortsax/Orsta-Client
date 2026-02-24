mod auth;
mod logger;
mod payment;
mod route;
mod schema;
mod sql;

use axum::Extension;
use payment::DummyPaymentProvider;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration, sleep};
use tracing::info;

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    let debug_mode = std::env::var("DEBUG_MODE")
        .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
        .unwrap_or(false);

    logger::init(debug_mode);

    let orchestrator = Arc::new(Mutex::new(sql::Orchestrator::init().await));

    info!("Orchestrator initialized. Ready to execute queries.");

    let sync_orch = Arc::clone(&orchestrator);
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(5)).await;
            let mut orch = sync_orch.lock().await;
            let ping = diesel::sql_query("SELECT 1");
            let _ = orch.sync_write(ping).await;
        }
    });

    let app = route::start_client_api_service(Arc::clone(&orchestrator));

    let dummy_mode = std::env::var("DUMMY_PAYMENT_MODE")
        .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
        .unwrap_or(false);

    let payment_provider: Arc<dyn payment::PaymentProvider> = if dummy_mode {
        info!("DUMMY_PAYMENT_MODE enabled — all payments will auto-succeed.");
        Arc::new(DummyPaymentProvider)
    } else {
        panic!("No PaymentProvider configured. Set DUMMY_PAYMENT_MODE=true for development or implement a real provider.");
    };

    let app = app.layer(Extension(payment_provider));

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    info!("WebSocket service started on {}", addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    info!("Server shut down cleanly.");
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for Ctrl+C");
    tracing::info!("Shutdown signal received.");
}
