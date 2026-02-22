mod schema;
mod sql;

#[tokio::main]
async fn main() {
    let orchestrator = sql::Orchestrator::init().await;

    println!("Orchestrator initialized. Ready to execute queries.");
}
