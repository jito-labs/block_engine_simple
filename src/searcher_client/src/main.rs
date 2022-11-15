use clap::Parser;

use std::{
    process::exit,
    sync::Arc,
    time::{Duration, Instant, SystemTime},
};

use jito_protos::searcher::searcher_service_client::SearcherServiceClient;
use jito_protos::{
    bundle::Bundle, proto_packet_from_versioned_tx, searcher::SendBundleRequest, shared::Header,
};
use log::{error, info};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::{CommitmentConfig, CommitmentLevel},
    pubkey::Pubkey,
    signature::{read_keypair_file, Signer},
    system_transaction,
    transaction::VersionedTransaction,
};
use tokio::runtime::Builder;
use tokio::time::sleep;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// RPC url to request airdrop from
    #[clap(short, long, env, default_value_t = String::from("http://localhost:8899"))]
    rpc_url: String,

    /// URL for searcher service
    #[clap(short, long, env, default_value_t = String::from("grpc://localhost:1234"))]
    searcher_service_url: String,

    /// Path to the keypair used to sign auth tokens.
    /// Ensure the associated pubkey is added to the auth store before running this script.
    #[clap(short, long, env, default_value_t = String::from("./keypair.json"))]
    keypair_path: String,
}

async fn request_and_confirm_airdrop(client: &RpcClient, pubkeys: &[Pubkey]) -> bool {
    let mut sigs = Vec::new();

    info!("requesting airdrop pubkeys: {:?}", pubkeys);

    for pubkey in pubkeys {
        let signature = client
            .request_airdrop(pubkey, 100000000000)
            .await
            .expect("gets signature");
        sigs.push(signature);
    }

    let now = Instant::now();
    while now.elapsed() < Duration::from_secs(20) {
        let r = client
            .get_signature_statuses(&sigs)
            .await
            .expect("got statuses");
        if r.value.iter().all(|s| s.is_some()) {
            info!("got airdrop pubkeys: {:?}", pubkeys);
            return true;
        }
    }
    false
}

fn main() {
    env_logger::init();

    let args: Args = Args::parse();

    let kp = Arc::new(read_keypair_file(args.keypair_path).expect("failed to read keypair file"));
    let rpc_client = RpcClient::new(args.rpc_url);

    let runtime = Builder::new_multi_thread().enable_all().build().unwrap();
    runtime.block_on(async move {
        let mut searcher_client = SearcherServiceClient::connect(args.searcher_service_url)
            .await
            .expect("connect to searcher service");
        if !request_and_confirm_airdrop(&rpc_client, &[kp.pubkey()]).await {
            error!("error requesting airdrop");
            exit(1);
        }
        sleep(Duration::from_secs(5)).await;

        let mut last_blockhash_time = Instant::now();
        let mut blockhash = rpc_client
            .get_latest_blockhash_with_commitment(CommitmentConfig {
                commitment: CommitmentLevel::Processed,
            })
            .await
            .expect("latest blockhash")
            .0;
        let mut base = 0;

        info!("sending bundles...");
        loop {
            if last_blockhash_time.elapsed() > Duration::from_secs(5) {
                blockhash = rpc_client
                    .get_latest_blockhash_with_commitment(CommitmentConfig {
                        commitment: CommitmentLevel::Processed,
                    })
                    .await
                    .expect("latest blockhash")
                    .0;
                last_blockhash_time = Instant::now();
            }
            let txs: Vec<_> = (0..5)
                .map(|amount| {
                    VersionedTransaction::from(system_transaction::transfer(
                        &kp,
                        &kp.pubkey(),
                        base + amount,
                        blockhash,
                    ))
                })
                .collect();
            base += txs.len() as u64;

            let result = searcher_client
                .send_bundle(SendBundleRequest {
                    bundle: Some(Bundle {
                        header: Some(Header {
                            ts: Some(prost_types::Timestamp::from(SystemTime::now())),
                        }),
                        packets: txs.iter().map(proto_packet_from_versioned_tx).collect(),
                    }),
                })
                .await;
            info!("uuid: {:?}", result.unwrap().into_inner().uuid);
            sleep(Duration::from_millis(1)).await;
        }
    });
}
