//! Handles the `getnettotals` RPC call.
//!
//! This RPC provides global network statistics from the Bitcoin node runtime:
//!
//! - Total bytes **sent** since node startup  
//! - Total bytes **received** since node startup  
//! - Upload target information  
//!
//! These metrics are useful for:
//! - Monitoring node bandwidth usage
//! - Debugging peers that saturate bandwidth
//! - Visualizing real-time network health and resource consumption
//!
//! The dashboard uses these values in the **Network** section to display
//! aggregated bandwidth behavior over time.

use reqwest::Client;
use reqwest::header::CONTENT_TYPE;
use serde_json::json;

use crate::models::network_totals::{NetTotalsJsonWrap, NetTotals};
use crate::models::errors::MyError;
use crate::config::RpcConfig;

use std::time::Duration;

/// Fetch total network byte counts using `getnettotals`.
///
/// ### Returns
/// A `NetTotals` struct containing:
/// - `totalbytesrecv` — cumulative bytes received  
/// - `totalbytessent` — cumulative bytes sent  
/// - Optional upload-target metadata  
///
/// The Bitcoin node tracks these counters from process start until shutdown.
///
/// ### Error Handling
/// Errors are converted to `MyError`:
/// - Timeout  
/// - Networking failure via Reqwest  
/// - JSON parsing issues  
///
/// ### Notes
/// - The call is lightweight and fast, safe to poll frequently.
/// - Counters grow monotonically; the dashboard can derive deltas if desired.
pub async fn fetch_net_totals(config: &RpcConfig) -> Result<NetTotals, MyError> {

    // Construct RPC request payload
    let json_rpc_request = json!({
        "jsonrpc": "1.0",
        "id": "1",
        "method": "getnettotals",
        "params": []
    });

    // Build HTTP client with sane timeouts
    let client = Client::builder()
        .timeout(Duration::from_secs(10))        // Max time for full RPC
        .connect_timeout(Duration::from_secs(5)) // Protect against stalled connections
        .build()?;

    // Execute RPC call
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
                    "Request to {} timed out for method 'getnettotals'",
                    config.address
                ))
            } else {
                MyError::Reqwest(e)
            }
        })?
        // Parse into wrapper which contains `.result: NetTotals`
        .json::<NetTotalsJsonWrap>()
        .await
        .map_err(|_e| {
            MyError::CustomError("JSON Parsing error for getnettotals.".to_string())
        })?;

    // Return deserialized network totals
    Ok(response.result)
}
