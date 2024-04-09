use std::net::SocketAddr;
use std::sync::Arc;

use ::capnp::capability::Promise;
use capnp_rpc::{pry, rpc_twoparty_capnp, twoparty, RpcSystem};
use futures::AsyncReadExt;

use peanutbutter::Service;

use crate::project_budget_capnp::project_budgets;
use project_budgets::{
    ExceedsBudgetParams, ExceedsBudgetResults, RecordSpendingParams, RecordSpendingResults,
};

pub async fn start_capnp(
    addr: SocketAddr,
    service: Arc<Service>,
) -> Result<(), Box<dyn std::error::Error>> {
    let service = CapnpService { inner: service };
    let client: project_budgets::Client = capnp_rpc::new_client(service);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    loop {
        let (stream, _) = listener.accept().await?;
        stream.set_nodelay(true)?;
        let (reader, writer) = tokio_util::compat::TokioAsyncReadCompatExt::compat(stream).split();
        let network = twoparty::VatNetwork::new(
            reader,
            writer,
            rpc_twoparty_capnp::Side::Server,
            Default::default(),
        );

        let rpc_system = RpcSystem::new(Box::new(network), Some(client.clone().client));
        tokio::task::spawn_local(rpc_system);
    }
}

#[derive(Debug)]
struct CapnpService {
    inner: Arc<Service>,
}

impl project_budgets::Server for CapnpService {
    fn exceeds_budget(
        &mut self,
        params: ExceedsBudgetParams,
        mut results: ExceedsBudgetResults,
    ) -> Promise<(), ::capnp::Error> {
        let request = pry!(pry!(params.get()).get_request());
        let config_name = pry!(pry!(request.get_config_name()).to_str());
        let project_id = request.get_project_id();

        let exceeds_budget = self.inner.exceeds_budget(config_name, project_id);

        results
            .get()
            .init_reply()
            .set_exceeds_budget(exceeds_budget);
        Promise::ok(())
    }

    fn record_spending(
        &mut self,
        params: RecordSpendingParams,
        mut results: RecordSpendingResults,
    ) -> Promise<(), ::capnp::Error> {
        let request = pry!(pry!(params.get()).get_request());
        let config_name = pry!(pry!(request.get_config_name()).to_str());
        let project_id = request.get_project_id();
        let spent = request.get_spent();

        let exceeds_budget = self.inner.record_spending(config_name, project_id, spent);

        results
            .get()
            .init_reply()
            .set_exceeds_budget(exceeds_budget);
        Promise::ok(())
    }
}
