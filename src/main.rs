// dev:
// 1. cargo install cargo-watch
// 2. cargo watch -x run

mod model;
mod open_rpc_server;
mod schedule;
mod service;

use crate::model::get_database;
use crate::schedule::start_schedules;
use dotenv::dotenv;
use ethers::contract::{abigen, Abigen, Contract};
use hyper::Method;
use jsonrpsee::server::{AllowHosts, ServerBuilder};
use mongodb::bson::doc;
use open_rpc_server::{OpenRpcServer, OpenRpcServerImpl};
use std::net::SocketAddr;
use tokio::fs::read_to_string;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // warning: only dev. The formal environment uses real env
    dotenv().ok();

    // Test db
    get_database()
        .await
        .run_command(doc! {"ping": 1}, None)
        .await?;

    start_schedules().await;

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
        .allow_methods([Method::POST, Method::OPTIONS])
        // Allow requests from any origin
        .allow_origin(tower_http::cors::Any)
        .allow_headers([hyper::header::CONTENT_TYPE]);
    let middleware = tower::ServiceBuilder::new().layer(cors);

    let bundler_rpc_host = std::env::var("BUNDLER_RPC_HOST").unwrap_or(String::from("127.0.0.1"));
    let bundler_rpc_port = std::env::var("BUNDLER_RPC_PORT").unwrap_or(String::from("4337"));

    // The RPC exposes the access control for filtering and the middleware for
    // modifying requests / responses. These features are independent of one another
    // and can also be used separately.
    // In this example, we use both features.
    let server = ServerBuilder::default()
        .set_host_filtering(AllowHosts::Any)
        .set_middleware(middleware)
        .build(format!("{}:{}", bundler_rpc_host, bundler_rpc_port).parse::<SocketAddr>()?)
        .await?;

    let addr = server.local_addr()?;
    let handle = server.start(OpenRpcServerImpl.into_rpc())?;

    println!("RpcServer started server on {}", addr);

    tokio::spawn(handle.stopped()).await?;

    Ok(addr)
}
