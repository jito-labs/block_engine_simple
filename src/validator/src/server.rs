use jito_protos::packet::PacketBatch;
use jito_protos::{
    block_engine::{
        block_engine_validator_server::BlockEngineValidator, BlockBuilderFeeInfoRequest,
        BlockBuilderFeeInfoResponse, SubscribeBundlesRequest, SubscribeBundlesResponse,
        SubscribePacketsRequest, SubscribePacketsResponse,
    },
    bundle::BundleUuid,
};
use log::{info, warn};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::{Builder, JoinHandle};
use tokio::sync::mpsc::error::TrySendError;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use uuid::Uuid;

pub struct AuthInterceptor {}

pub struct ValidatorServerImpl {
    forwarder_thread: JoinHandle<()>,
    packet_subscriptions:
        Arc<Mutex<HashMap<Uuid, Sender<Result<SubscribePacketsResponse, Status>>>>>,
    bundle_subscriptions:
        Arc<Mutex<HashMap<Uuid, Sender<Result<SubscribeBundlesResponse, Status>>>>>,
}

impl ValidatorServerImpl {
    pub fn new(
        bundle_receiver: Receiver<BundleUuid>,
        packet_receiver: Receiver<PacketBatch>,
    ) -> Self {
        let packet_subscriptions = Arc::new(Mutex::new(HashMap::default()));
        let bundle_subscriptions = Arc::new(Mutex::new(HashMap::default()));
        let forwarder_thread = Self::start_forwarder_thread(
            bundle_receiver,
            packet_receiver,
            &packet_subscriptions,
            &bundle_subscriptions,
        );
        Self {
            forwarder_thread,
            packet_subscriptions,
            bundle_subscriptions,
        }
    }

    pub fn join(self) -> thread::Result<()> {
        self.forwarder_thread.join()
    }

    fn start_forwarder_thread(
        mut bundle_receiver: Receiver<BundleUuid>,
        mut packet_receiver: Receiver<PacketBatch>,
        packet_subscriptions: &Arc<
            Mutex<HashMap<Uuid, Sender<Result<SubscribePacketsResponse, Status>>>>,
        >,
        bundle_subscriptions: &Arc<
            Mutex<HashMap<Uuid, Sender<Result<SubscribeBundlesResponse, Status>>>>,
        >,
    ) -> JoinHandle<()> {
        let packet_subscriptions = packet_subscriptions.clone();
        let bundle_subscriptions = bundle_subscriptions.clone();
        Builder::new()
            .name("forwarder_thread".into())
            .spawn(move || {
                let runtime = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .unwrap();
                runtime.block_on(async move {
                    loop {
                        tokio::select! {
                            maybe_packet_batch = packet_receiver.recv() => {
                                if let Some(packet_batch) = maybe_packet_batch {
                                    let failed_sends = Self::forward_packets(packet_batch, &packet_subscriptions).await;
                                    for uuid in failed_sends {
                                        info!("removing packet_subscriptions uuid: {:?}", uuid);
                                        packet_subscriptions.lock().unwrap().remove(&uuid);
                                    }
                                } else {
                                    warn!("packet_receiver disconnected, exiting");
                                    break;
                                }
                            }
                            maybe_bundle = bundle_receiver.recv() => {
                                if let Some(bundle) = maybe_bundle {
                                    let failed_sends = Self::forward_bundle(bundle, &bundle_subscriptions).await;
                                    for uuid in failed_sends {
                                        info!("removing bundle_subscriptions uuid: {:?}", uuid);
                                        bundle_subscriptions.lock().unwrap().remove(&uuid);
                                    }
                                } else {
                                    warn!("bundle_receiver disconnected, exiting");
                                    break;
                                }
                            }
                        }
                    }
                })
            })
            .unwrap()
    }

    async fn forward_packets(
        packet_batch: PacketBatch,
        packet_subscriptions: &Arc<
            Mutex<HashMap<Uuid, Sender<Result<SubscribePacketsResponse, Status>>>>,
        >,
    ) -> Vec<Uuid> {
        let mut failed_sends = Vec::new();
        let subs = packet_subscriptions.lock().unwrap();
        for (uuid, sender) in subs.iter() {
            match sender.try_send(Ok(SubscribePacketsResponse {
                header: None,
                batch: Some(packet_batch.clone()),
            })) {
                Ok(_) => {}
                Err(TrySendError::Closed(_)) => {
                    failed_sends.push(*uuid);
                }
                Err(TrySendError::Full(_)) => {
                    warn!("packet channel full uuid: {:?}", uuid);
                }
            }
        }
        failed_sends
    }

    async fn forward_bundle(
        bundle: BundleUuid,
        bundle_subscriptions: &Arc<
            Mutex<HashMap<Uuid, Sender<Result<SubscribeBundlesResponse, Status>>>>,
        >,
    ) -> Vec<Uuid> {
        let mut failed_sends = Vec::new();
        let subs = bundle_subscriptions.lock().unwrap();
        for (uuid, sender) in subs.iter() {
            match sender.try_send(Ok(SubscribeBundlesResponse {
                bundles: vec![bundle.clone()],
            })) {
                Ok(_) => {
                    info!("bundle forwarded validator uuid: {:?}", uuid);
                }
                Err(TrySendError::Closed(_)) => {
                    warn!("bundle channel closed validator uuid: {:?}", uuid);
                    failed_sends.push(*uuid);
                }
                Err(TrySendError::Full(_)) => {
                    warn!("bundle channel full validator uuid: {:?}", uuid);
                }
            }
        }
        failed_sends
    }
}

#[tonic::async_trait]
impl BlockEngineValidator for ValidatorServerImpl {
    type SubscribePacketsStream = ReceiverStream<Result<SubscribePacketsResponse, Status>>;

    async fn subscribe_packets(
        &self,
        _request: Request<SubscribePacketsRequest>,
    ) -> Result<Response<Self::SubscribePacketsStream>, Status> {
        let (sender, receiver) = channel(1000);

        let uuid = Uuid::new_v4();

        info!("adding packet_subscriptions uuid: {:?}", uuid);

        self.packet_subscriptions
            .lock()
            .unwrap()
            .insert(uuid, sender);

        Ok(Response::new(ReceiverStream::new(receiver)))
    }

    type SubscribeBundlesStream = ReceiverStream<Result<SubscribeBundlesResponse, Status>>;

    async fn subscribe_bundles(
        &self,
        _request: Request<SubscribeBundlesRequest>,
    ) -> Result<Response<Self::SubscribeBundlesStream>, Status> {
        let (sender, receiver) = channel(1000);

        let uuid = Uuid::new_v4();

        info!("adding bundle_subscriptions uuid: {:?}", uuid);

        self.bundle_subscriptions
            .lock()
            .unwrap()
            .insert(uuid, sender);

        Ok(Response::new(ReceiverStream::new(receiver)))
    }

    async fn get_block_builder_fee_info(
        &self,
        _request: Request<BlockBuilderFeeInfoRequest>,
    ) -> Result<Response<BlockBuilderFeeInfoResponse>, Status> {
        let response = BlockBuilderFeeInfoResponse {
            pubkey: Pubkey::default().to_string(),
            commission: 5,
        };

        info!("get_block_builder_fee_info response: {:?}", response);

        Ok(Response::new(BlockBuilderFeeInfoResponse {
            pubkey: Pubkey::default().to_string(),
            commission: 5,
        }))
    }
}
