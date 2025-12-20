//! Handles the `getchaintips` RPC call.
//!
//! This RPC returns all known chain tips on the node:
//! - The active chain tip
//! - Stale/orphaned tips
//! - Valid fork branches
//! - Unknown or invalid tips
//!
//! ### Why this matters
//! Fork monitoring is crucial for:
//! - Detecting stale blocks immediately
//! - Identifying deep reorgs
//! - Understanding network divergence
//! - Debugging unexpected validation or consensus issues
//!
//! The dashboard displays chain tips so operators can visually confirm
//! the health of the active chain and spot anomalies in real time.

use reqwest::Client;
use reqwest::header::CONTENT_TYPE;
use serde_json::json;

use crate::models::chaintips_info::{ChainTip, ChainTipsJsonWrap};
use crate::models::errors::MyError;
use crate::config::RpcConfig;

use std::time::Duration;

/// Fetch the list of known chain tips via `getchaintips`.
///
/// ### Returns
/// A `Vec<ChainTip>` containing tip metadata:
/// - `height`  
/// - `hash`  
/// - `branchlen`  
/// - `status` (`active`, `valid-fork`, `valid-headers`, `invalid`, etc.)  
///
/// ### Notes
/// - Bitcoin Core may return multiple tips even when the chain is healthy.
/// - The function does not perform any filtering — the caller decides how to use the results.
/// - The active chain tip always appears with `status = "active"`.
///
/// ### Error Handling
/// Produces `MyError` variants for:
/// - Timeout during RPC call  
/// - Transport/network issues  
/// - JSON parsing failures  
///
/// The function is intentionally simple and returns complete chain-tip data
/// exactly as the node reports it.
pub async fn fetch_chain_tips(config: &RpcConfig) -> Result<Vec<ChainTip>, MyError> {
    
    // Construct JSON-RPC request for getchaintips
    let json_rpc_request = json!({
        "jsonrpc": "1.0",
        "id": "1",
        "method": "getchaintips",
        "params": []
    });

    // Build HTTP client with conservative timeouts for fast refresh cycles
    let client = Client::builder()
        .timeout(Duration::from_secs(10))        // Limit total request time
        .connect_timeout(Duration::from_secs(5)) // Prevent hanging TCP handshakes
        .build()?;

    // Send request
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
                    "Request to {} timed out for method 'getchaintips'",
                    config.address
                ))
            } else {
                MyError::Reqwest(e)
            }
        })?
        // Deserialize into wrapper struct with `result: Vec<ChainTip>`
        .json::<ChainTipsJsonWrap>()
        .await
        .map_err(|_e| {
            MyError::CustomError("JSON Parsing error for getchaintips.".to_string())
        })?;

    Ok(response.result)
}
