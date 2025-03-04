
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
use std::time::Duration;

pub static MEMPOOL_CACHE: Lazy<Arc<DashSet<String>>> =
    Lazy::new(|| Arc::new(DashSet::new()));

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

    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(5))
        .build()?;

    let response = client
        .post(&config.address)
        .basic_auth(&config.username, Some(&config.password))
        .header(CONTENT_TYPE, "application/json")
        .json(&json_rpc_request)
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                MyError::TimeoutError(format!(
                    "Request to {} timed out for method 'getmempoolinfo'",
                    config.address
                ))
            } else {
                MyError::Reqwest(e)
            }
        })?
        .json::<MempoolInfoJsonWrap>()
        .await
        .map_err(|_e| {
            MyError::CustomError("JSON Parsing error for getmempoolinfo.".to_string())
        })?;

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
        .await
        .map_err(|e| {
            if e.is_timeout() {
                MyError::TimeoutError(format!(
                    "Request to {} timed out for method 'getrawmempool'",
                    config.address
                ))
            } else {
                MyError::Reqwest(e)
            }
        })?
        .json::<RawMempoolTxsJsonWrap>() 
        .await
        .map_err(|_e| {
            MyError::CustomError("JSON Parsing error for getrawmempool.".to_string())
        })?;

    // Clear the existing cache
    MEMPOOL_CACHE.clear();

    // Insert new transaction IDs into the cache
    for txid in raw_mempool_response.result {
        MEMPOOL_CACHE.insert(txid);
    }

    Ok(mempool_info)
}
