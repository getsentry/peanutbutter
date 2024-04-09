use std::sync::Arc;
use std::time::Duration;

use peanutbutter::*;

mod capnp;
mod grpc;

// XXX: this mod needs to be in the root package because the codegen assumes that
pub mod project_budget_capnp {
    include!(concat!(env!("OUT_DIR"), "/project_budget_capnp.rs"));
}

fn default_service() -> Arc<Service> {
    let backoff_duration = Duration::from_secs(5 * 60);
    let budgeting_window = Duration::from_secs(2 * 60);
    let bucket_size = Duration::from_secs(10);

    // TODO: we might want to have separate native/js budgets
    let allowed_budget = 5.0;

    let mut service = Service::new();

    service.add_config(
        "symbolication.native",
        BudgetingConfig::new(
            backoff_duration,
            budgeting_window,
            bucket_size,
            allowed_budget,
        ),
    );
    service.add_config(
        "symbolication.js",
        BudgetingConfig::new(
            backoff_duration,
            budgeting_window,
            bucket_size,
            allowed_budget,
        ),
    );

    Arc::new(service)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service = default_service();

    let grpc = grpc::start_grpc("0.0.0.0:9320".parse()?, service.clone());
    let capnp = capnp::start_capnp("0.0.0.0:9321".parse()?, service);

    println!("Starting grpc on localhost:9320");
    println!("Starting capnp on localhost:9321");

    let _ = tokio::join!(grpc, capnp);

    Ok(())
}
