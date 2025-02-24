
// rpc/mempool.rs

use reqwest::Client;
use reqwest::header::CONTENT_TYPE;
use serde_json::json;
use crate::models::mempool_info::{MempoolInfoJsonWrap, MempoolInfo, 
    RawMempoolTxsJsonWrap};
use crate::models::errors::MyError;
use crate::config::RpcConfig;
use std::sync::Arc;
use dashmap::DashSet;
use once_cell::sync::Lazy;
use tokio::sync::mpsc;

pub static MEMPOOL_CACHE: Lazy<Arc<DashSet<String>>> =
    Lazy::new(|| Arc::new(DashSet::new()));
    
// Fetches mempool information and samples raw transactions.
pub async fn fetch_mempool_info(
    config: &RpcConfig,
    tx: mpsc::Sender<Vec<String>>, // Add channel sender
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

    // Step 2: Fetch raw mempool transactions.
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

    // Clear the existing cache
    MEMPOOL_CACHE.clear();

    // Insert new transaction IDs into the cache
    let mut snapshot = Vec::new();
    for txid in raw_mempool_response.result {
        MEMPOOL_CACHE.insert(txid.clone());
        snapshot.push(txid); // Collect transaction IDs for the snapshot
    }

    // Send the snapshot to the distributor
    if tx.send(snapshot).await.is_err() {
        println!("Failed to send mempool snapshot");
    }

    Ok(mempool_info)
}