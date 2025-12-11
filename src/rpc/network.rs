//! Handles the `getnetworkinfo` RPC call.
//!
//! This RPC provides **node-level network metadata**, distinct from peer-specific data.
//! It supplies important operational details such as:
//!
//! - Node version / subversion
//! - Protocol version
//! - Local relay policies
//! - Connection limits (maxconnections, maxuploadtarget)
//! - Warnings
//! - Enabled/disabled networks (IPv4, IPv6, Onion)
//!
//! The dashboard uses this information to populate the **Network** section
//! and to give insight into how the node is configured and operating.

use reqwest::Client;
use reqwest::header::CONTENT_TYPE;
use serde_json::json;

use crate::models::network_info::{NetworkInfoJsonWrap, NetworkInfo};
use crate::models::errors::MyError;
use crate::config::RpcConfig;

use std::time::Duration;

/// Fetch high-level network metadata using `getnetworkinfo`.
///
/// ### Returns
/// A fully typed `NetworkInfo` struct containing:
/// - Node version and subversion  
/// - Whether the node is using full-relay or limited-relay settings  
/// - Current connection counts  
/// - Local services bitmask  
/// - Download/upload limits  
/// - Fee and relay policy settings  
/// - Network availability (IPv4, IPv6, TOR)  
///
/// ### Why This Matters
/// - The **Network** section of the dashboard displays this data directly.  
/// - Subversion parsing supports the *Client Distribution* chart.  
/// - Relay settings help determine whether fees or policies impact TX relay.  
///
/// ### Error Handling
/// Produces `MyError` variants for:
/// - Timeout  
/// - Reqwest network error  
/// - JSON parsing issues  
///
/// This call is safe to run frequently in the dashboard’s update loop.
pub async fn fetch_network_info(config: &RpcConfig) -> Result<NetworkInfo, MyError> {

    // Build RPC request for getnetworkinfo
    let json_rpc_request = json!({
        "jsonrpc": "1.0",
        "id": "1",
        "method": "getnetworkinfo",
        "params": []
    });

    // Build HTTP client with tight timeouts for TUI responsiveness
    let client = Client::builder()
        .timeout(Duration::from_secs(10))        // entire RPC timeout
        .connect_timeout(Duration::from_secs(5)) // TCP handshake timeout
        .build()?;

    // Execute RPC
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
                    "Request to {} timed out for method 'getnetworkinfo'",
                    config.address
                ))
            } else {
                MyError::Reqwest(e)
            }
        })?
        // Parse into wrapper struct containing NetworkInfo
        .json::<NetworkInfoJsonWrap>()
        .await
        .map_err(|_e| {
            MyError::CustomError("JSON Parsing error for getnetworkinfo.".to_string())
        })?;

    Ok(response.result)
}
