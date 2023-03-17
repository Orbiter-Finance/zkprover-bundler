use crate::model::get_database;
use ethers::types::{Transaction, H160, H256};
use mongodb::bson::DateTime;
use mongodb::Collection;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct PoolTx {
    pub tx: Transaction,
    pub tx_from: H160,
    pub tx_hash: H256,
    pub create_at: DateTime,
    pub status: u8, // 0: invalid, 1: received, 2: pending, 3: succeed, 4: failed
}

impl PoolTx {
    pub async fn get_collection() -> Collection<Self> {
        get_database().await.collection("pool_tx")
    }
}
