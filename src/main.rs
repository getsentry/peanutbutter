use std::time::Duration;

use tonic::{transport::Server, Request, Response, Status};

use proto::project_budgets_server::{ProjectBudgets, ProjectBudgetsServer};
use proto::{ExceedsBudgetReply, ExceedsBudgetRequest, RecordSpendingRequest};

mod proto {
    tonic::include_proto!("project_budget");
}

use peanutbutter::*;

fn default_service() -> Service {
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

    service
}

#[derive(Debug)]
struct GrpcService {
    inner: Service,
}

#[tonic::async_trait]
impl ProjectBudgets for GrpcService {
    async fn exceeds_budget(
        &self,
        request: Request<ExceedsBudgetRequest>,
    ) -> Result<Response<ExceedsBudgetReply>, Status> {
        let ExceedsBudgetRequest {
            config_name,
            project_id,
        } = request.into_inner();

        let exceeds_budget = self.inner.exceeds_budget(&config_name, project_id);

        Ok(Response::new(ExceedsBudgetReply { exceeds_budget }))
    }

    async fn record_spending(
        &self,
        request: Request<RecordSpendingRequest>,
    ) -> Result<Response<ExceedsBudgetReply>, Status> {
        let RecordSpendingRequest {
            config_name,
            project_id,
            spent,
        } = request.into_inner();

        let exceeds_budget = self.inner.record_spending(&config_name, project_id, spent);

        Ok(Response::new(ExceedsBudgetReply { exceeds_budget }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let addr = args.next().unwrap_or("0.0.0.0:50051".into());
    let addr = addr.parse()?;

    let service = GrpcService {
        inner: default_service(),
    };

    Server::builder()
        .add_service(ProjectBudgetsServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
