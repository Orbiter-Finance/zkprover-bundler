use crate::model::pool_batch::PoolBatch;
use crate::model::pool_tx::PoolTx;
use ethers::abi::AbiEncode;
use ethers::types::{Transaction, H256};
use ethers::utils::keccak256;
use futures::TryStreamExt;
use mongodb::bson::{doc, to_bson, DateTime};
use mongodb::options::FindOptions;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

pub async fn receive_tx(mut tx: Transaction) -> anyhow::Result<H256, anyhow::Error> {
    tx.from = tx.recover_from()?;
    tx.hash = tx.hash();

    let collection = PoolTx::get_collection().await;

    let one = collection
        .find_one(doc! {"tx_hash": tx.hash.encode_hex()}, None)
        .await?;

    if one.is_none() {
        let pool_tx = PoolTx {
            tx: tx.clone(),
            tx_from: tx.from,
            tx_hash: tx.hash,
            created_at: DateTime::from(SystemTime::now()),
            status: 1,
        };
        collection.insert_one(pool_tx, None).await?;
    }

    Ok(tx.hash)
}

pub async fn batch_received_txs() -> anyhow::Result<usize, anyhow::Error> {
    // const BATCH_TX_TOTAL: usize = 128;
    const BATCH_TX_TOTAL: usize = 1; // For dev

    let co_pool_tx = PoolTx::get_collection().await;
    let find_options = FindOptions::builder()
        .sort(doc! {"created_at": 1})
        .limit(BATCH_TX_TOTAL as i64)
        .build();
    let mut pt_cursor = co_pool_tx.find(doc! {"status": 1}, find_options).await?;

    let mut tx_hash_list: Vec<H256> = vec![];
    while let Some(tx) = pt_cursor.try_next().await? {
        tx_hash_list.push(tx.tx_hash);
    }

    // When received tx length >= BATCH_TX_TOTAL, new a batch
    if tx_hash_list.len() >= BATCH_TX_TOTAL {
        // Lock txs
        co_pool_tx
            .update_many(
                doc! {"tx_hash": {"$in": to_bson(&tx_hash_list)?}},
                doc! {"$set": {"status": 2}},
                None,
            )
            .await?;

        let batch_hash = keccak256(ethers::utils::rlp::encode_list(&tx_hash_list));

        let co_pool_batch = PoolBatch::get_collection().await;

        let pool_batch = PoolBatch {
            batch_hash: H256::from(batch_hash),
            tx_hash_list: tx_hash_list.clone(),
            zk_proof: None,
            zk_pub_input: None,
            created_at: DateTime::from(SystemTime::now()),
            status: 1,
        };
        co_pool_batch.insert_one(pool_batch, None).await?;
    }

    Ok(tx_hash_list.len())
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct GetPoolBatchResponse {
    batch_hash: H256,
    tx_list: Vec<Transaction>,
    status: u8,
}

pub async fn get_pool_batch() -> anyhow::Result<Option<GetPoolBatchResponse>, anyhow::Error> {
    let co_pool_tx = PoolTx::get_collection().await;
    let co_pool_batch = PoolBatch::get_collection().await;

    let pool_batch = co_pool_batch.find_one(doc! {"status": 1}, None).await?;
    match pool_batch {
        Some(pb) => {
            let mut pt_cursor = co_pool_tx
                .find(
                    doc! {"tx_hash": {"$in": to_bson(&pb.tx_hash_list).unwrap()}},
                    None,
                )
                .await?;

            let mut tx_list: Vec<Transaction> = vec![];
            while let Some(pt) = pt_cursor.try_next().await? {
                tx_list.push(pt.tx);
            }

            // Update batch status=2
            co_pool_batch
                .update_one(
                    doc! {"batch_hash": pb.batch_hash.encode_hex()},
                    doc! {"$set": {"status": 2}},
                    None,
                )
                .await?;

            Ok(Some(GetPoolBatchResponse {
                batch_hash: pb.batch_hash,
                tx_list,
                status: pb.status,
            }))
        }
        _ => Ok(None),
    }
}
