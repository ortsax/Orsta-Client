mod route;
mod schema;
mod sql;

use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration, sleep};

#[tokio::main]
async fn main() {
    let orchestrator = Arc::new(Mutex::new(sql::Orchestrator::init().await));

    println!("Orchestrator initialized. Ready to execute queries.");

    let sync_orch = Arc::clone(&orchestrator);
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(5)).await;
            let mut orch = sync_orch.lock().await;

            let ping = diesel::sql_query("SELECT 1");
            let _ = orch.sync_write(ping).await;

            println!("Background heartbeat sync performed.");
        }
    });

    let app = route::start_client_api_service(Arc::clone(&orchestrator));

    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    println!("🚀 API Service started on {}", addr);

    axum::serve(listener, app).await.unwrap();
}
