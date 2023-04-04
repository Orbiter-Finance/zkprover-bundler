use std::sync::Arc;
use std::time::SystemTime;

use ethers::abi;
use ethers::abi::{AbiEncode, ParamType};
use ethers::contract::abigen;
use ethers::middleware::SignerMiddleware;
use ethers::providers::Provider;
use ethers::signers::LocalWallet;
use ethers::types::{Bytes, Transaction, H160, H256, U256, U64};
use ethers::utils::keccak256;
use futures::TryStreamExt;
use mongodb::bson::{doc, to_bson, DateTime};
use mongodb::options::FindOptions;
use serde::{Deserialize, Serialize};
use tokio::task;

use crate::model::pool_batch::PoolBatch;
use crate::model::pool_tx::PoolTx;
use crate::schedule::do_batch_received_txs;

abigen!(EntryPointContract, "./src/config/contracts/EntryPoint.json");

async fn handle_ops(pb: PoolBatch) -> anyhow::Result<H256, anyhow::Error> {
    let co_pool_tx = PoolTx::get_collection().await;
    let co_pool_batch = PoolBatch::get_collection().await;

    println!("Do handle_ops: {}", pb.batch_hash.encode_hex());

    let mut ops: Vec<UserOperation> = vec![];
    for h in pb.tx_hash_list.iter() {
        let one = co_pool_tx
            .find_one(doc! {"tx_hash": h.encode_hex()}, None)
            .await?;

        if one.is_none() {
            continue;
        }

        let pool_tx = one.unwrap();
        let input_data = pool_tx.tx.input.clone();
        let (_, decoded_input_data) = input_data.split_at(4);

        let tokens = abi::decode(
            &[
                ParamType::Array(Box::new(ParamType::Tuple(vec![
                    ParamType::Address,
                    ParamType::Uint(256),
                    ParamType::Bytes,
                    ParamType::Bytes,
                    ParamType::Uint(256),
                    ParamType::Uint(256),
                    ParamType::Uint(256),
                    ParamType::Uint(256),
                    ParamType::Uint(256),
                    ParamType::Bytes,
                    ParamType::Bytes,
                ]))),
                ParamType::Bytes,
                ParamType::FixedArray(Box::new(ParamType::Uint(256)), 1),
                ParamType::Address,
            ],
            &decoded_input_data,
        )
        .unwrap();

        let tuple_arr = tokens[0].clone().into_array().unwrap();
        for tuple in tuple_arr.iter() {
            let t = tuple.clone().into_tuple().unwrap();
            ops.push(UserOperation {
                sender: t[0].clone().into_address().unwrap(),
                nonce: t[1].clone().into_uint().unwrap(),
                init_code: Bytes::from(t[2].clone().into_bytes().unwrap()),
                call_data: Bytes::from(t[3].clone().into_bytes().unwrap()),
                call_gas_limit: t[4].clone().into_uint().unwrap(),
                verification_gas_limit: t[5].clone().into_uint().unwrap(),
                pre_verification_gas: t[6].clone().into_uint().unwrap(),
                max_fee_per_gas: t[7].clone().into_uint().unwrap(),
                max_priority_fee_per_gas: t[8].clone().into_uint().unwrap(),
                paymaster_and_data: Bytes::from(t[9].clone().into_bytes().unwrap()),
                signature: Bytes::from(t[10].clone().into_bytes().unwrap()),
            });
        }
    }

    let miner_private_key = std::env::var("BUNDLER_MINER_PRIVATE_KEY").unwrap();
    let miner_address: H160 = std::env::var("BUNDLER_MINER_ADDRESS").unwrap().parse()?;
    let entry_point_address: H160 = std::env::var("BUNDLER_ENTRY_POINT_ADDRESS")
        .unwrap()
        .parse()?;

    let wallet: LocalWallet = miner_private_key.parse::<LocalWallet>()?;

    let provider =
        Provider::<ethers::providers::Http>::try_from(std::env::var("NETWORK_RPC_URL").unwrap())
            .unwrap();

    let client =
        SignerMiddleware::new_with_provider_chain(provider.clone(), wallet.clone()).await?;

    let entry_point =
        EntryPointContract::new(entry_point_address.clone(), Arc::new(client.clone()));

    // let proof = pb.zk_proof.unwrap_or("0x00".parse().unwrap());
    let proof = "0x0bd25a842f34fbdb111a96c6a27cf6bd29a917915d70807f59c22ca17b32d60b02dbeaf4b49cf47989c526b142401585d4369acabae337ec828ef0d202e93ddb13041248ff33e20bdb0aeb53b9202d2a57c24ce4545d7280cea041506f2c887120f7eeb1c6876810364caa21108fcbdb8c0adae0255423cb6f6e3891d7cad7b909058994926a6a0c776c264e0e1590278a158140e452a59851eef753576106d029990ca0e49f5e2e327ad2c797bbc8ba4b3551741b7417964ecae8495ba4c1be2915b39125efa2ce148e5a92c9101fd3a321656017343f3f3be2b061b5b5cfe812d4ceec3d4f53d863b80d5caca99375c6b54bf46fa91a8761f75e20e06ac7d41f1fbcc8f1b470be0ceaf408ce73f7d1f17c5d6f4f534c24f741d0a510c085b126041332b3b6262595e4b064e97874703bbd40f3cd7aa31af1bc19ed294969b4184e6b705c4cf411016036e8f51ce8848ab419ce3c63006091970b1bcdd03c611df617ba23c70af6df78d4cf26c4103aeef33d826cbe0a63738a0e879cc64a4c0701c8dda4f0cc5156bb2e592abc9b62aae0ca86eeff207686ab0268b869387e18ead49e65976f8239c61ac2cb1f5fb743cf707e6f49f10a6e26a2022dbd81f510b91b47890dfa23be5d8cba526e3bbc9efe3bb126c36bbf3113e8fd72b75b792edb90eda6fcea504906b55a7dda7bcc8543a806a38ed37c65ac933fb2cd98a0200cfb6d46def7096c452262286cf3771d6e95c784c80734e4f6e2440914ef5b11713ea681b4856ece02d478fb340a70c7e97ed1540a5113b9f39f28cbf7d7dd14e634613867782602890c7c6e9cdf0b74ad55e65c69170f4a29ed9d2088915c19a3f987615dd78ef36d1a27215d70f19213f588136dbd58c8a2b1b46813ad611b16a64c0f47b974d26517e8954aaafbc946ce2150f518d5939e1ba932ab62a32a061ddde12d4afe906c68238bbb184794f8633b40d2c61a5b9766d115df2fbd15355a8affe474874a4082cb9db0903af6d814e5cc8905d25ed1b1b75a8229130552ae3789943969fb7f7f3de44f1dc30c91bde5e8f19c891f121a621733220e1c4da3bae02bbaa191111370d57da4f8edf14bd8d4488402ff50ac84520c28f41a0011183889c29807daf60ca9ef982aaa22ff855603916233caff1ae86a718111bea0a12ca6c6d3fbdc38a23d7c322baff97ce4c95e339705ff3e9696cf90b609830f08deec3b9bded2bc6ebf4225d7d2e3b67e3137c9d1bc94aba431716219277cd8544a2040b44a392ef1ec090b79b40fac486b8818de8c7c89aefff42cf52972823f1e82a78d1940d666ef4b50f782188c03832b828d13ada74e942a2e9a253fecda3850a978cf7559b797c5ac67ffc3a2d03f55b38d4548be4b8cd55a210c08f9941c8dd52bb802f35b9f368ff7d3ce92283de8cde6e77b9107cfb85b7117a787905a857339d1f74c50d85d412fd55341a8e7042d36ce78f6c4dfec3e410ffc065422f563659de15dbbd62be0abb4d71753ac27c0a80c3b48cf395539c2049530e954cafb7622ec8f853770878850a2b1d09be5eaf58d78c57cac6b96c10837057be2b551f80e0a79914fa9cd820683ed6d838e4d8a2f7ec1c4817524621d206dfb7e0c58a46bbed69f6d75967797f64b5b7c40abc791018091deffae6e0b8829cf362e7393cc512876ebc781c62e95660aabbb0e868201c0d9b0feda33234714a55c5f87b4cdb77c91839ca45b77ca3820d1a4b973c8ba696fd1ee4f201297af1d35c87e6ecbdd1d3e9ffe3a345c56c8b4d47a2a07943212f88c29e29f22d7558723df6ba871a6b6af09529ca5bf1b4467e0e4f4c8cb30a016a687dfd313940cd3b2da338f4343895b396f25f2ba3be2d51d2677b3d663a043c2b55ae9045ebcf595b672edcd1967751237b7b200d283764acacf97122222b0842b91cb11892d919307e40f6cb9d6d2a9dc5c408f7e67c6b40ddbfce8efa891cddc08882c9da7a008592a006f287d1318482f3ef8f05bc1df9bd929c1cae7fa1e8eaba70d2de3179b52e644015e2ff7d1da43aa167cf113c2ad01f148639dc9803750ed".parse::<Bytes>().unwrap_or("0x00".parse().unwrap());
    let pub_signals: Vec<U256> = pb.zk_pub_inputs.clone();
    println!("proof:{}", proof.clone().encode_hex());
    println!("pub_signals:{:#?}", pub_signals.clone());

    let transaction_receipt = entry_point
        .handle_ops(ops, proof, [pub_signals[0]], miner_address)
        .gas(2000000)
        .send()
        .await?
        .await?;

    match transaction_receipt {
        Some(tr) => {
            let send_tx_hash = tr.transaction_hash.encode_hex();
            println!("send_tx_hash: {}", send_tx_hash);

            co_pool_batch
                .update_one(
                    doc! {"batch_hash": &pb.batch_hash.encode_hex()},
                    doc! {"$set": {"status": 4, "send_tx_hash": send_tx_hash }},
                    None,
                )
                .await?;
            Ok(tr.transaction_hash)
        }
        _ => {
            co_pool_batch
                .update_one(
                    doc! {"batch_hash": &pb.batch_hash.encode_hex()},
                    doc! {"$set": {"status": 5}},
                    None,
                )
                .await?;
            Ok(H256::zero())
        }
    }
}

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

    tokio::spawn(do_batch_received_txs());

    Ok(tx.hash)
}

pub async fn batch_received_txs() -> anyhow::Result<usize, anyhow::Error> {
    let bundler_batch_tx_total: usize = std::env::var("BUNDLER_BATCH_TX_TOTAL")
        .unwrap_or(String::from("128"))
        .parse()
        .unwrap();

    let co_pool_tx = PoolTx::get_collection().await;
    let find_options = FindOptions::builder()
        .sort(doc! {"created_at": 1})
        .limit(bundler_batch_tx_total as i64)
        .build();
    let mut pt_cursor = co_pool_tx.find(doc! {"status": 1}, find_options).await?;

    let mut tx_hash_list: Vec<H256> = vec![];
    while let Some(tx) = pt_cursor.try_next().await? {
        tx_hash_list.push(tx.tx_hash);
    }

    // When received tx length >= bundler_batch_tx_total, new a batch
    if tx_hash_list.len() >= bundler_batch_tx_total {
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
            zk_pub_inputs: vec![],
            send_tx_hash: H256::zero(),
            created_at: DateTime::from(SystemTime::now()),
            status: 1,
        };
        co_pool_batch.insert_one(pool_batch, None).await?;
    }

    Ok(tx_hash_list.len())
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
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

pub async fn receive_proof_and_public_input(
    batch_hash: H256,
    zk_proof: Bytes,
    zk_pub_inputs: Vec<U256>,
) -> anyhow::Result<U64, anyhow::Error> {
    let co_pool_batch = PoolBatch::get_collection().await;
    let hash_encode_hex = batch_hash.encode_hex();

    let pool_batch = co_pool_batch
        .find_one(
            doc! {"batch_hash": hash_encode_hex.as_str(), "status": {"$in": [1, 2]}},
            None,
        )
        .await?;

    match pool_batch {
        Some(_) => {
            co_pool_batch
                .update_one(
                    doc! {"batch_hash": hash_encode_hex.as_str()},
                    doc! {"$set": {"zk_proof": zk_proof.encode_hex(), "zk_pub_inputs": to_bson(&zk_pub_inputs).unwrap(), "status": 3}},
                    None,
                )
                .await?;

            task::spawn(async move {
                let pool_batch = co_pool_batch
                    .find_one(doc! {"batch_hash": hash_encode_hex.as_str()}, None)
                    .await
                    .unwrap();
                match pool_batch {
                    Some(pool_batch) => {
                        handle_ops(pool_batch).await.unwrap();
                    }
                    _ => {}
                }
            });

            Ok(U64::from(1))
        }
        _ => Ok(U64::from(0)),
    }
}
