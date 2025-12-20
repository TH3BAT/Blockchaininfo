//! Handles the `getpeerinfo` RPC call.
//!
//! This RPC provides detailed metadata for every connected peer:
//! - Version / subversion (used for Client Distribution)
//! - Inbound / outbound connection flags
//! - Ping time (used for propagation insights)
//! - Local and remote addresses
//! - Services and relay flags
//!
//! The data returned here powers:
//! - Network view panel
//! - Client Distribution chart
//! - Block propagation timing metrics
//!
//! Because `getpeerinfo` can return a large array depending on the node,
//! this module uses strict timeouts to ensure UI responsiveness.

use crate::models::peer_info::{PeerInfo, PeerInfoJsonWrap};
use crate::config::RpcConfig;
use crate::models::errors::MyError;

use reqwest::{Client, header::CONTENT_TYPE};
use serde_json::json;
use std::time::Duration;

/// Fetch peer metadata using `getpeerinfo`.
///
/// ### Returns
/// A vector of `PeerInfo` objects, each representing a connected peer.
/// These contain rich metadata including:
/// - Version / Subversion  
/// - Ping times  
/// - Inbound/outbound direction  
/// - Relay flags  
/// - Connection age  
///
/// ### Uses in the Dashboard
/// - **Client Distribution**  
///   Parses `subver` strings to determine which node software and versions peers are running.
///
/// - **Propagation Time Measurements**  
///   Ping times and last block receive times help identify slow or lagging peers.
///
/// ### Error Handling
/// Returns `MyError` variants for:
/// - Timeout  
/// - Network transport failure  
/// - JSON deserialization error  
///
/// This function is performance-sensitive since it may be called on each refresh loop.
pub async fn fetch_peer_info(config: &RpcConfig) -> Result<Vec<PeerInfo>, MyError> {

    // Build JSON-RPC request
    let json_rpc_request = json!({
        "jsonrpc": "1.0",
        "id": "1",
        "method": "getpeerinfo",
        "params": []
    });

    // Lightweight RPC client with conservative timeouts
    let client = Client::builder()
        .timeout(Duration::from_secs(10))        // Entire RPC must finish within 10s
        .connect_timeout(Duration::from_secs(5)) // Avoid hanging on connection attempts
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
                    "Request to {} timed out for method 'getpeerinfo'",
                    config.address
                ))
            } else {
                MyError::Reqwest(e)
            }
        })?
        // Deserialize into wrapper struct containing `result: Vec<PeerInfo>`
        .json::<PeerInfoJsonWrap>()
        .await
        .map_err(|_e| {
            MyError::CustomError("JSON Parsing error for getpeerinfo.".to_string())
        })?;

    Ok(response.result)
}
