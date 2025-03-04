
// rpc/network_peers.rs

use crate::models::peer_info::{PeerInfo, PeerInfoResponse};
use crate::config::RpcConfig;
use crate::models::errors::MyError;
use reqwest::{Client, header::CONTENT_TYPE};
use serde_json::json;
use std::time::Duration;

pub async fn fetch_peer_info(config: &RpcConfig) -> Result<Vec<PeerInfo>, MyError> {
    let json_rpc_request = json!({
        "jsonrpc": "1.0",
        "id": "1",
        "method": "getpeerinfo",
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
                    "Request to {} timed out for method 'getpeerinfo'",
                    config.address
                ))
            } else {
                MyError::Reqwest(e)
            }
        })?
        .json::<PeerInfoResponse>()
        .await
        .map_err(|_e| {
            MyError::CustomError("JSON Parsing error for getpeerinfo.".to_string())
        })?;

    Ok(response.result)
}