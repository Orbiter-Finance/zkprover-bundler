use std::net::SocketAddr;
use std::time::Duration;

use hyper::body::Bytes;
use hyper::Method;
use jsonrpsee::core::client::ClientT;
use jsonrpsee::http_client::HttpClientBuilder;
use jsonrpsee::rpc_params;
use jsonrpsee::server::{AllowHosts, RpcModule, ServerBuilder};
use tower_http::cors::CorsLayer;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tower_http::LatencyUnit;
use tracing_subscriber::filter::Directive;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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
        .build("127.0.0.1:8090".parse::<SocketAddr>()?)
        .await?;

    let mut module = RpcModule::new(());

    module.register_method("say_hello", |_, _| {
        println!("say_hello method called!");
        Ok("Hello there!!")
    })?;

    module.register_method("eth_chainId", |_, _| Ok("0x5"))?;
    module.register_method("net_version", |_, _| Ok("0x5"))?;

    module.register_method("eth_getBalance", |_, _| Ok("0x63fff10634435b5"))?;

    module.register_method("eth_blockNumber", |_, _| Ok("0x83980a"))?;

    module.register_method("eth_getBlockByNumber", |_, _| {
        let data = "{\"baseFeePerGas\":\"0xc0da4cb67\",\"difficulty\":\"0x0\",\"extraData\":\"0x4e65746865726d696e64\",\"gasLimit\":\"0x1c9c380\",\"gasUsed\":\"0x1c973c1\",\"hash\":\"0x1d6cfba5d352c7fb313170bab07eb38e222b3f481899c5b5f4761bd38e09d8b6\",\"logsBloom\":\"0x86be3215f5e0d6dbc5d1091586c92991cd3e75053a51fa92a2b0a91d2ef1515e66c615a4bb8942131313d15493c588ddb1a6b6b26ffe7bfe9e00574c686666aa7ee9e10dde7aa0dcc89b427b783cb324498d343343c5a20cd8f6c9a5928ca2005e1fbfeefa8545cc52600ea783048f663dd0eaf92ffe49cfb0fd7575e61a8a5c538ef3b5f8cfa53aac1d5c36a2e541ea58cb958d2bf2560f3975f57110a08366222588517508f7da9720b52e7eb2766f7305c683cc14b15493b30f47a7b01fc62fb05102138eff0dbd2390a70523ed3c93fc5a9349e8c5994ad325ec7c966b02d8ff3e3f2be4024aad27d448f393cc1cf0b34eba19983641bb570fb0f2c0484a\",\"miner\":\"0x4d496ccc28058b1d74b7a19541663e21154f9c84\",\"mixHash\":\"0xb13411cdcdb7cc61275fad4c6ebc1586375b493a621c174e17da52ccead6ec15\",\"nonce\":\"0x0000000000000000\",\"number\":\"0x839894\",\"parentHash\":\"0xf6e29a347b68852ecaf82b18b5c603a7023ec532ccf49658184557fb29ac65ed\",\"receiptsRoot\":\"0x9e6daef5e6478bc7cc713c757840f0afa9728f43ed06c974ea6106a93382cb64\",\"sha3Uncles\":\"0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347\",\"size\":\"0x316a8\",\"stateRoot\":\"0x917964ea4f85bb76c672e8091dbf7406d11c4bbd5eb02d262a2da0cde110b4c4\",\"timestamp\":\"0x6409c32c\",\"totalDifficulty\":\"0xa4a470\",\"transactions\":[\"0x3e7783f2fdd03035cdd0c7d629de1cb839a29609050da4758d9ea352d0a98d16\"],\"transactionsRoot\":\"0x7f5b603db6e5ac71f194bfc4202fa71d5e90c338a4acf2de70c43eab97a3ea7f\",\"uncles\":[]}";
        let value = serde_json::from_str(data)?;
        Ok(value)
    })?;

    module.register_method("eth_getCode", |_, _| Ok("0x"))?;

    module.register_method("eth_gasPrice", |_, _| Ok("0x20"))?;

    module.register_method("eth_call", |_, _| Ok("0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000063fff10634435b50000000000000000000000000000000000000000000000000823f6b920ad4752000000000000000000000000000000000000000000000007aad7588eb796333d"))?;

    let addr = server.local_addr()?;
    let handle = server.start(module)?;

    tokio::spawn(handle.stopped()).await?;

    Ok(addr)
}
