// cargo install cargo-watch
// cargo watch -x run

mod rpc_server;

use std::net::SocketAddr;
use std::time::Duration;

use dotenv::dotenv;
use ethers::prelude::SignerMiddleware;
use ethers::providers::{Middleware, Provider};
use ethers::signers::{LocalWallet, Wallet};
use ethers::types::{BlockId, Chain};
use hyper::body::Bytes;
use hyper::Method;
use jsonrpsee::core::client::ClientT;
use jsonrpsee::http_client::HttpClientBuilder;
use jsonrpsee::rpc_params;
use jsonrpsee::server::{AllowHosts, RpcModule, ServerBuilder};
use rpc_server::{EthRpcServer, EthRpcServerImpl};
use serde_json::Value;
use tower_http::cors::CorsLayer;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tower_http::LatencyUnit;
use tracing_subscriber::filter::Directive;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // warnning: only dev. The formal environment uses real env
    dotenv().ok();

    // let provider = Provider::<ethers::providers::Http>::try_from(
    //     "https://eth-goerli.g.alchemy.com/v2/rS3DfLJRdaQyZAJSvj8lYK_9rwWhpeGV",
    // )?;

    // let chainid = provider.get_chainid().await?;
    // println!("chainid: {}", chainid);

    // let wallet: LocalWallet = ""
    //     .parse::<LocalWallet>()?;
    // // .with_chain_id(Chain::Goerli);

    // let client = SignerMiddleware::new(provider.clone(), wallet.clone());

    // let block_id = provider.get_block_number().await;
    // let from = "0x6ce4D9694c1626862234216bA78874dE70903A71";
    // let balance = client.get_balance(from, None).await?;

    // println!("balance: {}", balance);

    tracing_subscriber::FmtSubscriber::builder()
        // .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init()
        .expect("setting default subscriber failed");

    run_server().await?;

    Ok(())
}

async fn run_server() -> anyhow::Result<SocketAddr> {
    // Add a CORS middleware for handling HTTP requests.
    // This middleware does affect the response, including appropriate
    // headers to satisfy CORS. Because any origins are allowed, the
    // "Access-Control-Allow-Origin: *" header is appended to the response.
    let cors = CorsLayer::new()
        // Allow `POST` when accessing the resource
        .allow_methods([Method::POST, Method::GET, Method::OPTIONS])
        // Allow requests from any origin
        .allow_origin(tower_http::cors::Any)
        .allow_headers([hyper::header::CONTENT_TYPE]);
    let middleware = tower::ServiceBuilder::new().layer(cors);

    // The RPC exposes the access control for filtering and the middleware for
    // modifying requests / responses. These features are independent of one another
    // and can also be used separately.
    // In this example, we use both features.
    let server = ServerBuilder::default()
        .set_host_filtering(AllowHosts::Any)
        .set_middleware(middleware)
        .build("127.0.0.1:8546".parse::<SocketAddr>()?)
        .await?;

    let addr = server.local_addr()?;
    let handle = server.start(EthRpcServerImpl.into_rpc())?;

    tokio::spawn(handle.stopped()).await?;

    Ok(addr)
}
