use ethers::types::transaction::eip2718::TypedTransaction;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct BundlerList {
    pub tx: TypedTransaction,
    pub status: u8, // 0: receive, 1: pedding, 2: succeed, 3: failed
}
