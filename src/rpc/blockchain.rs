//! Handles the `getblockchaininfo` RPC call.
//!
//! This RPC provides chain-wide metadata required across the dashboard:
//! - Current chain height
//! - Difficulty
//! - Chainwork
//! - Verification progress
//! - Block/headers count
//! - IBD (initial block download) status
//! - Network name (main, test, regtest)
//!
//! This is one of the most frequently called RPCs in the application and forms
//! the foundation for difficulty calculations, epoch analysis, and UI displays.

use reqwest::header::CONTENT_TYPE;
use serde_json::json;

use crate::models::blockchain_info::{BlockchainInfoJsonWrap, BlockchainInfo};
use crate::models::errors::MyError;
use crate::config::RpcConfig;
use crate::rpc::client::build_rpc_client;

/// Fetches blockchain-wide metadata via `getblockchaininfo`.
///
/// ### Returns
/// A fully typed `BlockchainInfo` struct containing:
/// - Current block height  
/// - Difficulty  
/// - Chainwork  
/// - Headers count  
/// - IBD state  
/// - Verification progress  
///
/// ### RPC Details
/// Method: **getblockchaininfo**  
/// Params: *none*  
///
/// ### Error Handling
/// Converts underlying errors into `MyError`:
/// - Timeout during RPC call  
/// - Reqwest network failure  
/// - JSON parse failure (malformed or unexpected data)  
///
/// This function is called continuously by the main application loop and must
/// remain fast, stable, and predictable.
pub async fn fetch_blockchain_info(config: &RpcConfig) -> Result<BlockchainInfo, MyError> {
    
    // Construct raw JSON-RPC request payload
    let json_rpc_request = json!({
        "jsonrpc": "1.0",
        "id": "1",
        "method": "getblockchaininfo",
        "params": []
    });

    // Configure lightweight RPC client with tight timeouts for TUI responsiveness
    let client = build_rpc_client()?;

    // Execute request
    let response = client
        .post(&config.address)
        .basic_auth(&config.username, Some(&config.password))
        .header(CONTENT_TYPE, "application/json")
        .json(&json_rpc_request)
        .send()
        .await
        .map_err(|e| {
            // Distinguish timeout from other network errors
            if e.is_timeout() {
                MyError::TimeoutError(format!(
                    "Request to {} timed out for method 'getblockchaininfo'",
                    config.address
                ))
            } else {
                MyError::Reqwest(e)
            }
        })?
        // Deserialize into wrapper type containing a `result: BlockchainInfo`
        .json::<BlockchainInfoJsonWrap>()
        .await
        .map_err(|_e| {
            MyError::CustomError("JSON Parsing error for getblockchaininfo.".to_string())
        })?;

    Ok(response.result)
}
