use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::{Json, State};
use axum::routing::post;
use axum::Router;
use serde::{Deserialize, Serialize};

use peanutbutter::*;

fn default_service() -> Service {
    let backoff_duration = Duration::from_secs(5 * 60);
    let budgeting_window = Duration::from_secs(2 * 60);
    let bucket_size = Duration::from_secs(10);

    let mut service = Service::new();

    service.add_config(
        "symbolication-native",
        BudgetingConfig::new(backoff_duration, budgeting_window, bucket_size, 5.0),
    );
    service.add_config(
        "symbolication-js",
        BudgetingConfig::new(backoff_duration, budgeting_window, bucket_size, 5.0),
    );

    service.add_config(
        "symbolication-jvm",
        BudgetingConfig::new(backoff_duration, budgeting_window, bucket_size, 7.5),
    );

    service
}

#[derive(Deserialize)]
struct RecordSpendingRequest {
    config_name: String,
    project_id: u64,
    spent: f64,
}

#[derive(Deserialize)]
struct ExceedsBudgetRequest {
    config_name: String,
    project_id: u64,
}

#[derive(Serialize)]
struct ExceedsBudgetResponse {
    exceeds_budget: bool,
}

async fn record_spending(
    State(service): State<Arc<Service>>,
    Json(request): Json<RecordSpendingRequest>,
) -> Json<ExceedsBudgetResponse> {
    let exceeds_budget =
        service.record_spending(&request.config_name, request.project_id, request.spent);
    Json(ExceedsBudgetResponse { exceeds_budget })
}

async fn exceeds_budget(
    State(service): State<Arc<Service>>,
    Json(request): Json<ExceedsBudgetRequest>,
) -> Json<ExceedsBudgetResponse> {
    let exceeds_budget = service.exceeds_budget(&request.config_name, request.project_id);
    Json(ExceedsBudgetResponse { exceeds_budget })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let addr = args.next().unwrap_or("0.0.0.0:4433".into());
    let addr: SocketAddr = addr.parse()?;

    let service = Arc::new(default_service());

    let app = Router::new()
        .route("/record_spending", post(record_spending))
        .route("/exceeds_budget", post(exceeds_budget))
        .with_state(service);

    println!("Starting server on `{addr}`…");
    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, app).await?;

    Ok(())
}
