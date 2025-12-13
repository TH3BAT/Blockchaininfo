//! Handles mempool-related RPC calls.
//!
//! This module is responsible for:
//! - Fetching high-level mempool statistics (`getmempoolinfo`)
//! - Fetching the complete list of mempool transaction IDs (`getrawmempool`)
//! - Maintaining a global, thread-safe mempool TXID cache (`MEMPOOL_CACHE`)
//!
//! The global cache is consumed by the mempool distribution system and other
//! modules that require awareness of which transactions currently reside in the mempool.

use reqwest::Client;
use reqwest::header::CONTENT_TYPE;
use serde_json::json;

use crate::models::mempool_info::{
    MempoolInfoJsonWrap,
    MempoolInfo,
    RawMempoolTxsJsonWrap
};
use crate::models::errors::MyError;
use crate::config::RpcConfig;

use std::sync::Arc;
use dashmap::DashSet;
use once_cell::sync::Lazy;
use std::time::Duration;
use hex::FromHex;

/// Global mempool TXID cache.
///
/// Stores every TXID currently in the node's mempool (as returned by `getrawmempool`).
///
/// ### Why DashSet?
/// - Thread-safe concurrent reads/writes  
/// - Fast membership checks  
/// - Ideal for the mempool distribution engine and toggles  
///
/// This cache is **rebuilt** on every call to `fetch_mempool_info`
/// to ensure it reflects the latest state from the node.
pub static MEMPOOL_CACHE: Lazy<Arc<DashSet<[u8; 32]>>> =
    Lazy::new(|| Arc::new(DashSet::new()));

/// Fetches mempool statistics and the full list of mempool transaction IDs.
///
/// ### Steps Performed
///
/// #### **1. Fetch mempool info (`getmempoolinfo`)**
/// Provides:
/// - Total TX count  
/// - Mempool size  
/// - Min fee  
/// - Maxmempool settings  
/// - Usage/bytes  
///
/// This is returned to the caller of the function.
///
/// #### **2. Fetch raw mempool transaction IDs (`getrawmempool`)**
/// - Called with `false` to return **only TXIDs**, not detailed entries.
/// - This ensures a lightweight call that scales well.
///
/// #### **3. Rebuild the global TXID cache**
/// - Clears old entries  
/// - Inserts all new TXIDs atomically  
///
/// ### Error Handling
/// Errors are converted to `MyError`:
/// - Timeout  
/// - Network failure  
/// - JSON deserialization failure  
///
/// ### Consumers
/// - `mempool_distro.rs` (distribution engine)  
/// - Dashboard mempool charts  
/// - Any module needing a list of active TXIDs  
pub async fn fetch_mempool_info(
    config: &RpcConfig,
) -> Result<MempoolInfo, MyError> {
    
    // ─────────────────────────────────────────────────────────────
    // Step 1: Fetch mempool info (high-level mempool statistics)
    // ─────────────────────────────────────────────────────────────
    let json_rpc_request = json!({
        "jsonrpc": "1.0",
        "id": "1",
        "method": "getmempoolinfo",
        "params": []
    });

    let client = Client::builder()
        .timeout(Duration::from_secs(10))        // Full request timeout
        .connect_timeout(Duration::from_secs(5)) // TCP handshake timeout
        .build()?;

    let mempoolinfo_response = client
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

    // ─────────────────────────────────────────────────────────────
    // Step 2: Fetch raw mempool TXIDs
    // ─────────────────────────────────────────────────────────────
    let json_rpc_request = json!({
        "jsonrpc": "1.0",
        "id": "2",
        "method": "getrawmempool",
        "params": [false] // return only transaction IDs
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

    // ─────────────────────────────────────────────────────────────
    // Step 3: Rebuild the global mempool TXID cache
    // ─────────────────────────────────────────────────────────────
    MEMPOOL_CACHE.clear();

    raw_mempool_response
        .result
        .iter()
        .filter_map(|txid| txid_hex_to_bytes(txid))
        .for_each(|txid_bytes| {
            MEMPOOL_CACHE.insert(txid_bytes);
        });

    // Return the parsed mempool info struct
    Ok(mempoolinfo_response.result)
}



fn txid_hex_to_bytes(txid: &str) -> Option<[u8; 32]> {
    let vec = Vec::from_hex(txid).ok()?;
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&vec);
    Some(arr)
}