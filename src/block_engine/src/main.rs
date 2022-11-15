use clap::Parser;
use jito_auth::server::AuthServiceImpl;
use jito_protos::auth::auth_service_server::AuthServiceServer;
use jito_protos::block_engine::block_engine_validator_server::BlockEngineValidatorServer;
use jito_protos::searcher::searcher_service_server::SearcherServiceServer;
use jito_searcher::server::SearcherServiceImpl;
use jito_validator::server::ValidatorServerImpl;
use log::info;
use std::net::SocketAddr;
use tokio::runtime::Builder;
use tokio::sync::mpsc::channel;
use tonic::transport::Server;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Bind address for searcher service
    #[clap(long, env, default_value = "0.0.0.0:1234")]
    searcher_addr: SocketAddr,

    /// Bind address for validator service
    #[clap(long, env, default_value = "0.0.0.0:1003")]
    validator_addr: SocketAddr,

    /// Bind address for validator service
    #[clap(long, env, default_value = "0.0.0.0:1005")]
    auth_addr: SocketAddr,
}

fn main() {
    env_logger::init();

    let args: Args = Args::parse();

    let (_packet_sender, packet_receiver) = channel(100);
    let (bundle_sender, bundle_receiver) = channel(100);

    let runtime = Builder::new_multi_thread().enable_all().build().unwrap();
    runtime.block_on(async move {
        // start searcher server
        tokio::spawn(async move {
            let searcher_service_impl = SearcherServiceImpl::new(bundle_sender);
            let searcher_svc = SearcherServiceServer::new(searcher_service_impl);
            info!("starting searcher server at {}", args.searcher_addr);
            Server::builder()
                .add_service(searcher_svc)
                .serve(args.searcher_addr)
                .await
                .expect("searcher server starts");
        });

        // start auth server
        tokio::spawn(async move {
            let auth_service_impl = AuthServiceImpl::new();
            let auth_svc = AuthServiceServer::new(auth_service_impl);
            info!("starting auth server at {}", args.auth_addr);
            Server::builder()
                .add_service(auth_svc)
                .serve(args.auth_addr)
                .await
                .expect("auth server starts");
        });

        // start validator server and block
        let validator_impl = ValidatorServerImpl::new(bundle_receiver, packet_receiver);
        let validator_svc = BlockEngineValidatorServer::new(validator_impl);
        info!("starting validator server at {}", args.validator_addr);
        Server::builder()
            .add_service(validator_svc)
            .serve(args.validator_addr)
            .await
            .expect("validator server starts");
    });
}
