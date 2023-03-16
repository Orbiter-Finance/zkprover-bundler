use ethers::types::Transaction;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct BundlerList {
    pub tx: Transaction,
    pub status: u8, // 0: receive, 1: pedding, 2: succeed, 3: failed
}
