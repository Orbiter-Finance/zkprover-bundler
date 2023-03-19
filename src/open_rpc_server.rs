use ethers::providers::{Middleware, Provider};
use ethers::types::{
    transaction::eip2718::TypedTransaction, Address, Block, BlockId, Bytes, NameOrAddress, H256,
    U256, U64,
};
use ethers::types::{BlockNumber, FeeHistory, Transaction, TransactionReceipt};
use jsonrpsee::core::{async_trait, RpcResult};
use jsonrpsee::proc_macros::rpc;
use serde_json::{json, Value};

use crate::service::pool;
use crate::service::pool::GetPoolBatchResponse;

fn get_http_provider() -> Provider<ethers::providers::Http> {
    Provider::<ethers::providers::Http>::try_from(std::env::var("NETWORK_RPC_URL").unwrap())
        .unwrap()
}

fn json_to_typed_transaction(json: Value) -> Result<TypedTransaction, serde_json::error::Error> {
    let mut clone_json = json.clone();
    if let Some(m) = clone_json.as_object_mut() {
        if let Some(m_type) = m.get_mut("type") {
            match m_type.as_str().unwrap_or("0x0") {
                "0x0" => *m_type = json!("0x00"),
                "0x1" => *m_type = json!("0x01"),
                "0x2" => *m_type = json!("0x02"),
                &_ => {}
            }
        } else {
            m.entry("type").or_insert(json!("0x00"));
        }
    }

    serde_json::from_value(clone_json)
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
    async fn eth_get_code(
        &self,
        address: NameOrAddress,
        block_id: Option<BlockId>,
    ) -> RpcResult<Bytes>;

    #[method(name = "eth_gasPrice")]
    async fn eth_gas_price(&self) -> RpcResult<U256>;

    #[method(name = "eth_feeHistory")]
    async fn eth_fee_history(
        &self,
        block_count: U256,
        last_block: BlockNumber,
        reward_percentiles: Vec<f64>,
    ) -> RpcResult<FeeHistory>;

    #[method(name = "eth_call")]
    async fn eth_call(&self, tx: Value, block: Option<BlockId>) -> RpcResult<Bytes>;

    #[method(name = "eth_estimateGas")]
    async fn eth_estimate_gas(&self, tx: Value, block: Option<BlockId>) -> RpcResult<U256>;

    #[method(name = "eth_getTransactionCount")]
    async fn eth_get_transaction_count(
        &self,
        from: NameOrAddress,
        block: Option<BlockId>,
    ) -> RpcResult<U256>;

    #[method(name = "eth_sendRawTransaction")]
    async fn eth_send_raw_transaction(&self, tx: Bytes) -> RpcResult<H256>;

    #[method(name = "eth_getTransactionReceipt")]
    async fn eth_get_transaction_receipt(
        &self,
        transaction_hash: H256,
    ) -> RpcResult<Option<TransactionReceipt>>;

    #[method(name = "zkp_getPoolBatch")]
    async fn zkp_get_pool_batch(&self) -> RpcResult<Option<GetPoolBatchResponse>>;

    #[method(name = "zkp_sendProofAndPublicInput")]
    async fn zkp_send_proof_and_public_inputs(
        &self,
        batch_hash: H256,
        zk_proof: Bytes,
        zk_pub_inputs: Vec<U256>,
    ) -> RpcResult<U64>;
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

    async fn eth_get_code(&self, at: NameOrAddress, block_id: Option<BlockId>) -> RpcResult<Bytes> {
        let provider = get_http_provider();
        let result = provider.get_code(at, block_id).await;

        match result {
            Ok(result) => Ok(result),
            Err(error) => Err(jsonrpsee::core::Error::Custom(error.to_string())),
        }
    }

    async fn eth_gas_price(&self) -> RpcResult<U256> {
        let provider = get_http_provider();
        let result = provider.get_gas_price().await;

        match result {
            Ok(result) => Ok(result),
            Err(error) => Err(jsonrpsee::core::Error::Custom(error.to_string())),
        }
    }

    async fn eth_fee_history(
        &self,
        block_count: U256,
        last_block: BlockNumber,
        reward_percentiles: Vec<f64>,
    ) -> RpcResult<FeeHistory> {
        let provider = get_http_provider();
        let result = provider
            .fee_history(block_count, last_block, &reward_percentiles)
            .await;

        match result {
            Ok(result) => Ok(result),
            Err(error) => Err(jsonrpsee::core::Error::Custom(error.to_string())),
        }
    }

    async fn eth_call(&self, tx: Value, block: Option<BlockId>) -> RpcResult<Bytes> {
        // Data cleaning
        let ttx = json_to_typed_transaction(tx)?;

        let provider = get_http_provider();
        let result = provider.call(&ttx, block).await;

        match result {
            Ok(result) => Ok(result),
            Err(error) => Err(jsonrpsee::core::Error::Custom(error.to_string())),
        }
    }

    async fn eth_estimate_gas(&self, tx: Value, block: Option<BlockId>) -> RpcResult<U256> {
        // Data cleaning
        let ttx = json_to_typed_transaction(tx)?;

        let provider = get_http_provider();
        let result = provider.estimate_gas(&ttx, block).await;

        match result {
            Ok(result) => Ok(result),
            Err(error) => Err(jsonrpsee::core::Error::Custom(error.to_string())),
        }
    }

    async fn eth_get_transaction_count(
        &self,
        from: NameOrAddress,
        block: Option<BlockId>,
    ) -> RpcResult<U256> {
        let provider = get_http_provider();
        let result = provider.get_transaction_count(from, block).await;

        match result {
            Ok(result) => Ok(result),
            Err(error) => Err(jsonrpsee::core::Error::Custom(error.to_string())),
        }
    }

    async fn eth_send_raw_transaction(&self, raw_tx: Bytes) -> RpcResult<H256> {
        let tx: Transaction = ethers::utils::rlp::decode(&raw_tx).unwrap();
        let result = pool::receive_tx(tx).await;

        match result {
            Ok(result) => Ok(result),
            Err(error) => Err(jsonrpsee::core::Error::Custom(error.to_string())),
        }
    }

    async fn eth_get_transaction_receipt(
        &self,
        transaction_hash: H256,
    ) -> RpcResult<Option<TransactionReceipt>> {
        let provider = get_http_provider();
        let result = provider.get_transaction_receipt(transaction_hash).await;

        match result {
            Ok(result) => Ok(result),
            Err(error) => Err(jsonrpsee::core::Error::Custom(error.to_string())),
        }
    }

    async fn zkp_get_pool_batch(&self) -> RpcResult<Option<GetPoolBatchResponse>> {
        let result = pool::get_pool_batch().await;

        match result {
            Ok(result) => Ok(result),
            Err(error) => Err(jsonrpsee::core::Error::Custom(error.to_string())),
        }
    }

    async fn zkp_send_proof_and_public_inputs(
        &self,
        batch_hash: H256,
        zk_proof: Bytes,
        zk_pub_inputs: Vec<U256>,
    ) -> RpcResult<U64> {
        let result =
            pool::receive_proof_and_public_input(batch_hash, zk_proof, zk_pub_inputs).await;

        match result {
            Ok(result) => Ok(result),
            Err(error) => Err(jsonrpsee::core::Error::Custom(error.to_string())),
        }
    }
}
