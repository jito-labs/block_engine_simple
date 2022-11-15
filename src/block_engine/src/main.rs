use clap::Parser;
use jito_protos::searcher::searcher_service_server::SearcherServiceServer;
use jito_searcher::server::SearcherServiceImpl;
use log::info;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::runtime::Builder;
use tokio::sync::mpsc::channel;
use tokio::time::sleep;
use tonic::transport::Server;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Use json_logger unless this is set to true, then use env_logger
    #[clap(long, env)]
    searcher_addr: SocketAddr,
}

fn main() {
    env_logger::init();

    let args: Args = Args::parse();

    let (bundle_sender, bundle_receiver) = channel(100);

    let runtime = Builder::new_multi_thread().enable_all().build().unwrap();
    runtime.block_on(async move {
        tokio::spawn(async move {
            let searcher_service_impl = SearcherServiceImpl::new(bundle_sender);
            let svc = SearcherServiceServer::new(searcher_service_impl);
            info!("starting searcher server at {}", args.searcher_addr);
            Server::builder()
                .add_service(svc)
                .serve(args.searcher_addr)
                .await
                .expect("server to start");
        });
    });
}
