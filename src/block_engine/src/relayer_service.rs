use jito_protos::block_engine::{
    block_engine_relayer_server::BlockEngineRelayer, AccountsOfInterestRequest,
    AccountsOfInterestUpdate, PacketBatchUpdate, ProgramsOfInterestRequest,
    ProgramsOfInterestUpdate, StartExpiringPacketStreamResponse,
};
use jito_protos::shared::Heartbeat;
use log::info;
use std::pin::Pin;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::Stream;
use tonic::{Request, Response, Status};

pub struct RelayerServiceImpl {}

impl RelayerServiceImpl {
    pub fn new() -> Self {
        RelayerServiceImpl {}
    }
}

#[tonic::async_trait]
impl BlockEngineRelayer for RelayerServiceImpl {
    type SubscribeAccountsOfInterestStream =
        Pin<Box<dyn Stream<Item = Result<AccountsOfInterestUpdate, Status>> + Send + Sync>>;

    async fn subscribe_accounts_of_interest(
        &self,
        req: Request<AccountsOfInterestRequest>,
    ) -> Result<Response<Self::SubscribeAccountsOfInterestStream>, Status> {
        info!("BlockEngineRelayer::subscribe_accounts_of_interest called from {:?}", req.remote_addr());

        let (tx, rx) = mpsc::channel(10);

        // Spawn a task that keeps the stream alive with periodic updates
        tokio::spawn(async move {
            let update = AccountsOfInterestUpdate {
                accounts: vec!["*".to_string()],
            };
            info!("Sending initial AccountsOfInterestUpdate with wildcard");
            let _ = tx.send(Ok(update)).await;

            // Keep the stream alive by holding the sender
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
                // Send periodic update to keep stream alive
                let update = AccountsOfInterestUpdate {
                    accounts: vec!["*".to_string()],
                };
                if tx.send(Ok(update)).await.is_err() {
                    info!("AccountsOfInterest stream closed");
                    break;
                }
            }
        });

        Ok(Response::new(Box::pin(ReceiverStream::new(rx))))
    }

    type SubscribeProgramsOfInterestStream =
        Pin<Box<dyn Stream<Item = Result<ProgramsOfInterestUpdate, Status>> + Send + Sync>>;

    async fn subscribe_programs_of_interest(
        &self,
        req: Request<ProgramsOfInterestRequest>,
    ) -> Result<Response<Self::SubscribeProgramsOfInterestStream>, Status> {
        info!("BlockEngineRelayer::subscribe_programs_of_interest called from {:?}", req.remote_addr());

        let (tx, rx) = mpsc::channel(10);

        // Spawn a task that keeps the stream alive with periodic updates
        tokio::spawn(async move {
            let update = ProgramsOfInterestUpdate {
                programs: vec!["*".to_string()],
            };
            info!("Sending initial ProgramsOfInterestUpdate with wildcard");
            let _ = tx.send(Ok(update)).await;

            // Keep the stream alive by holding the sender
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
                // Send periodic update to keep stream alive
                let update = ProgramsOfInterestUpdate {
                    programs: vec!["*".to_string()],
                };
                if tx.send(Ok(update)).await.is_err() {
                    info!("ProgramsOfInterest stream closed");
                    break;
                }
            }
        });

        Ok(Response::new(Box::pin(ReceiverStream::new(rx))))
    }

    type StartExpiringPacketStreamStream = Pin<
        Box<dyn Stream<Item = Result<StartExpiringPacketStreamResponse, Status>> + Send + Sync>,
    >;

    async fn start_expiring_packet_stream(
        &self,
        req: Request<tonic::Streaming<PacketBatchUpdate>>,
    ) -> Result<Response<Self::StartExpiringPacketStreamStream>, Status> {
        info!("BlockEngineRelayer::start_expiring_packet_stream called from {:?}", req.remote_addr());

        let mut in_stream = req.into_inner();
        let (tx, rx) = mpsc::channel(10);

        // Spawn a task to consume the incoming stream and send heartbeats back
        tokio::spawn(async move {
            let mut count = 0u64;

            // Send initial heartbeat
            let heartbeat = StartExpiringPacketStreamResponse {
                heartbeat: Some(Heartbeat { count }),
            };
            info!("Sending initial heartbeat (count={})", count);
            let _ = tx.send(Ok(heartbeat)).await;

            // Consume incoming packets (just log them)
            while let Ok(Some(_msg)) = in_stream.message().await {
                count += 1;
                info!("Received packet batch update #{}", count);
                // Send a heartbeat response periodically
                let heartbeat = StartExpiringPacketStreamResponse {
                    heartbeat: Some(Heartbeat { count }),
                };
                info!("Sending heartbeat response (count={})", count);
                let _ = tx.send(Ok(heartbeat)).await;
            }
            info!("start_expiring_packet_stream ended");
        });

        Ok(Response::new(Box::pin(ReceiverStream::new(rx))))
    }
}
