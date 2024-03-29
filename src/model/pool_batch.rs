use crate::model::get_database;
use ethers::types::{Bytes, H256, U256};
use mongodb::bson::DateTime;
use mongodb::Collection;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct PoolBatch {
    pub batch_hash: H256,
    pub tx_hash_list: Vec<H256>,
    pub zk_proof: Option<Bytes>,
    pub zk_pub_inputs: Vec<U256>,
    pub send_tx_hash: H256,
    pub created_at: DateTime,
    pub status: u8, // 0: invalid, 1: received, 2: pending, 3: submitting, 4: succeed, 5: failed
}

impl PoolBatch {
    pub async fn get_collection() -> Collection<Self> {
        get_database().await.collection("pool_batch")
    }
}
