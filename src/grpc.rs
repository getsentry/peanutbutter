use std::net::SocketAddr;
use std::sync::Arc;

use tonic::{transport::Server, Request, Response, Status};

use proto::project_budgets_server::{ProjectBudgets, ProjectBudgetsServer};
use proto::{ExceedsBudgetReply, ExceedsBudgetRequest, RecordSpendingRequest};

use peanutbutter::Service;

mod proto {
    tonic::include_proto!("project_budget");
}

pub async fn start_grpc(
    addr: SocketAddr,
    service: Arc<Service>,
) -> Result<(), Box<dyn std::error::Error>> {
    let service = GrpcService { inner: service };

    Server::builder()
        .add_service(ProjectBudgetsServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}

#[derive(Debug)]
struct GrpcService {
    inner: Arc<Service>,
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
