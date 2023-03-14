use ethers::providers::{Middleware, Provider};
use ethers::types::{Address, Block, BlockId, H256, U256, U64};
use jsonrpsee::core::{async_trait, RpcResult};
use jsonrpsee::proc_macros::rpc;

fn get_http_provider() -> Provider<ethers::providers::Http> {
    Provider::<ethers::providers::Http>::try_from(
        "https://eth-goerli.g.alchemy.com/v2/rS3DfLJRdaQyZAJSvj8lYK_9rwWhpeGV",
    )
    .unwrap()
}

#[rpc(server)]
pub trait OpenRpc {
    #[method(name = "net_version")]
    async fn net_version(&self) -> RpcResult<String>;

    #[method(name = "eth_chainId")]
    async fn eth_chainid(&self) -> RpcResult<String>;

    #[method(name = "eth_getBalance")]
    async fn eth_get_balance(&self, address: Address, block_id: Option<BlockId>)
        -> RpcResult<U256>;

    #[method(name = "eth_blockNumber")]
    async fn eth_block_number(&self) -> RpcResult<U64>;

    #[method(name = "eth_getBlockByNumber")]
    async fn eth_get_block_by_number(
        &self,
        block_id: Option<BlockId>,
    ) -> RpcResult<Option<Block<H256>>>;

    #[method(name = "eth_getCode")]
    async fn eth_get_code(&self) -> RpcResult<&'static str>;

    #[method(name = "eth_gasPrice")]
    async fn eth_gas_price(&self) -> RpcResult<&'static str>;

    #[method(name = "eth_feeHistory")]
    async fn eth_fee_history(&self) -> RpcResult<Vec<&'static str>>;

    #[method(name = "eth_call")]
    async fn eth_call(&self) -> RpcResult<&'static str>;

    #[method(name = "eth_estimateGas")]
    async fn eth_estimate_gas(&self) -> RpcResult<&'static str>;

    #[method(name = "eth_sendRawTransaction")]
    async fn eth_send_raw_transaction(&self) -> RpcResult<&'static str>;
}

pub struct OpenRpcServerImpl;

#[async_trait]
impl OpenRpcServer for OpenRpcServerImpl {
    async fn net_version(&self) -> RpcResult<String> {
        Ok(std::env::var("BUNDLER_CHAINID").unwrap())
    }

    async fn eth_chainid(&self) -> RpcResult<String> {
        Ok(std::env::var("BUNDLER_CHAINID").unwrap())
    }

    async fn eth_get_balance(
        &self,
        address: Address,
        block_id: Option<BlockId>,
    ) -> RpcResult<U256> {
        let provider = get_http_provider();
        let result = provider.get_balance(address, block_id).await;

        match result {
            Ok(result) => Ok(result),
            Err(error) => Err(jsonrpsee::core::Error::Custom(error.to_string())),
        }
    }

    async fn eth_block_number(&self) -> RpcResult<U64> {
        let provider = get_http_provider();
        let result = provider.get_block_number().await;

        match result {
            Ok(result) => Ok(result),
            Err(error) => Err(jsonrpsee::core::Error::Custom(error.to_string())),
        }
    }

    async fn eth_get_block_by_number(
        &self,
        block_id: Option<BlockId>,
    ) -> RpcResult<Option<Block<H256>>> {
        let provider = get_http_provider();
        let result = provider.get_block(block_id.unwrap()).await;

        match result {
            Ok(result) => Ok(result),
            Err(error) => Err(jsonrpsee::core::Error::Custom(error.to_string())),
        }
    }

    async fn eth_get_code(&self) -> RpcResult<&'static str> {
        Ok("0x")
    }

    async fn eth_gas_price(&self) -> RpcResult<&'static str> {
        Ok("0x20")
    }

    async fn eth_fee_history(&self) -> RpcResult<Vec<&'static str>> {
        Ok(vec!["0x20"])
    }

    async fn eth_call(&self) -> RpcResult<&'static str> {
        Ok("0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000063fff10634435b50000000000000000000000000000000000000000000000000823f6b920ad4752000000000000000000000000000000000000000000000007aad7588eb796333d")
    }

    async fn eth_estimate_gas(&self) -> RpcResult<&'static str> {
        Ok("0x10010")
    }

    async fn eth_send_raw_transaction(&self) -> RpcResult<&'static str> {
        Ok("0x")
    }
}
