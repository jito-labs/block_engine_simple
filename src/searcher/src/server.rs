use jito_protos::bundle::{BundleResult, BundleUuid};
use jito_protos::searcher::{
    searcher_service_server::SearcherService, ConnectedLeadersRegionedRequest,
    ConnectedLeadersRegionedResponse, ConnectedLeadersRequest, ConnectedLeadersResponse,
    GetRegionsRequest, GetRegionsResponse, GetTipAccountsRequest, GetTipAccountsResponse,
    NextScheduledLeaderRequest, NextScheduledLeaderResponse, SendBundleRequest, SendBundleResponse,
    SubscribeBundleResultsRequest,
};
use log::info;
use tokio::sync::mpsc::Sender;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use uuid::Uuid;

pub struct SearcherServiceImpl {
    bundle_sender: Sender<BundleUuid>,
}

impl SearcherServiceImpl {
    pub const MAX_BUNDLE_LEN: usize = 5;

    #[allow(clippy::too_many_arguments)]
    pub fn new(bundle_sender: Sender<BundleUuid>) -> Self {
        SearcherServiceImpl { bundle_sender }
    }
}

#[tonic::async_trait]
impl SearcherService for SearcherServiceImpl {
    type SubscribeBundleResultsStream = ReceiverStream<Result<BundleResult, Status>>;

    async fn subscribe_bundle_results(
        &self,
        _request: Request<SubscribeBundleResultsRequest>,
    ) -> Result<Response<Self::SubscribeBundleResultsStream>, Status> {
        unimplemented!()
    }

    async fn send_bundle(
        &self,
        request: Request<SendBundleRequest>,
    ) -> Result<Response<SendBundleResponse>, Status> {
        info!("SearcherService::send_bundle called from {:?}", request.remote_addr());

        let uuid = Uuid::new_v4().to_string();
        let bundle_uuid = BundleUuid {
            bundle: request.into_inner().bundle,
            uuid: uuid.clone(),
        };

        info!("received bundle_uuid: {:?}", bundle_uuid.uuid);

        if bundle_uuid.bundle.is_some() {
            self.bundle_sender
                .send(bundle_uuid)
                .await
                .map_err(|_| Status::internal("error forwarding bundle"))?;
        }

        Ok(Response::new(SendBundleResponse { uuid }))
    }

    async fn get_next_scheduled_leader(
        &self,
        _request: Request<NextScheduledLeaderRequest>,
    ) -> Result<Response<NextScheduledLeaderResponse>, Status> {
        unimplemented!()
    }

    async fn get_connected_leaders(
        &self,
        _request: Request<ConnectedLeadersRequest>,
    ) -> Result<Response<ConnectedLeadersResponse>, Status> {
        unimplemented!()
    }

    async fn get_connected_leaders_regioned(
        &self,
        _request: Request<ConnectedLeadersRegionedRequest>,
    ) -> Result<Response<ConnectedLeadersRegionedResponse>, Status> {
        unimplemented!()
    }

    async fn get_tip_accounts(
        &self,
        _request: Request<GetTipAccountsRequest>,
    ) -> Result<Response<GetTipAccountsResponse>, Status> {
        unimplemented!()
    }

    async fn get_regions(
        &self,
        _request: Request<GetRegionsRequest>,
    ) -> Result<Response<GetRegionsResponse>, Status> {
        unimplemented!()
    }
}
