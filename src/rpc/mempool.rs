
// rpc/mempool.rs

use reqwest::Client;
use reqwest::header::CONTENT_TYPE;
use serde_json::json;
use crate::models::mempool_info::{MempoolInfoJsonWrap, MempoolInfo, 
    RawMempoolTxsJsonWrap};
use crate::models::errors::MyError;
use crate::config::RpcConfig;
use std::sync::{Arc, RwLock};
use std::collections::HashSet;

pub static MEMPOOL_CACHE: once_cell::sync::Lazy<Arc<RwLock<HashSet<String>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(RwLock::new(HashSet::new())));

// Fetches mempool information and samples raw transactions.
pub async fn fetch_mempool_info(
    config: &RpcConfig,
) -> Result<MempoolInfo, MyError> {
    // Step 1: Fetch mempool information (to get the transaction count).
    let json_rpc_request = json!({
        "jsonrpc": "1.0",
        "id": "1",
        "method": "getmempoolinfo",
        "params": []
    });

    let client = Client::new();
    let response = client
        .post(&config.address)
        .basic_auth(&config.username, Some(&config.password))
        .header(CONTENT_TYPE, "application/json")
        .json(&json_rpc_request)
        .send()
        .await?
        .json::<MempoolInfoJsonWrap>()
        .await?;

    let mempool_info = response.result;

    // Step 3: Fetch raw mempool transactions.
    let json_rpc_request = json!({
        "jsonrpc": "1.0",
        "id": "2",
        "method": "getrawmempool",
        "params": [false] // false to return transaction IDs only.
    });

    let raw_mempool_response = client
        .post(&config.address)
        .basic_auth(&config.username, Some(&config.password))
        .header(CONTENT_TYPE, "application/json")
        .json(&json_rpc_request)
        .send()
        .await?
        .json::<RawMempoolTxsJsonWrap>() 
        .await?;

    // Store raw mempool TXs in a fast, shared HashSet.
    let mut cache = MEMPOOL_CACHE.write().unwrap();
    *cache = raw_mempool_response.result.into_iter().collect();

    Ok(mempool_info)
}
